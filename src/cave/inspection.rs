//! Read-only inspection of ELF64 binaries.
//!
//! None of the functions here modify the binary. They are intended as a
//! pre-injection scouting step — find out what space already exists before
//! deciding whether to inject a new segment at all.

use object::Endianness;
use object::elf::*;
use object::read::elf::{ElfFile64, FileHeader, ProgramHeader, SectionHeader};

use crate::cave::binary::resolve_shstrndx;
use crate::elf::ElfFile;
use crate::error::{CaverError, Result};

/// A single ELF64 section.
#[derive(Debug, Clone)]
pub struct SectionInfo {
    /// Section name, resolved from `.shstrtab`.
    pub name: String,
    /// Virtual memory address (`sh_addr`), zero for non-allocated sections.
    pub vma: u64,
    /// File offset (`sh_offset`).
    pub offset: u64,
    /// Size in bytes (`sh_size`).
    pub size: u64,
    /// Section type (`sh_type`), e.g. `SHT_PROGBITS`.
    pub sh_type: u32,
    /// Section flags (`sh_flags`).
    pub sh_flags: u64,
}

impl SectionInfo {
    /// Returns true if the section is marked executable (SHF_EXECINSTR).
    pub fn is_executable(&self) -> bool {
        self.sh_flags & (SHF_EXECINSTR as u64) != 0
    }

    /// Returns true if the section is marked writable (SHF_WRITE).
    pub fn is_writable(&self) -> bool {
        self.sh_flags & (SHF_WRITE as u64) != 0
    }
}

impl std::fmt::Display for SectionInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} vma={:#x} offset={:#x} size={} type={:#x} flags={:#x}",
            self.name, self.vma, self.offset, self.size, self.sh_type, self.sh_flags
        )
    }
}

/// Returns all sections in `elf`, in section-header-table order.
pub(crate) fn list_sections(elf: &ElfFile) -> Result<Vec<SectionInfo>> {
    let parsed = ElfFile64::<Endianness>::parse(elf.data.as_slice())?;
    let endian = parsed.endian();
    let elf_header = parsed.elf_header();

    let sections: &[SectionHeader64<Endianness>] =
        elf_header.section_headers(endian, elf.data.as_slice())?;

    let shstrndx = resolve_shstrndx(endian, elf_header, sections)?;
    let shstrtab_sh = sections.get(shstrndx).ok_or(CaverError::NotElf64)?;
    let shstrtab_off = shstrtab_sh.sh_offset(endian) as usize;
    let shstrtab_sz = shstrtab_sh.sh_size(endian) as usize;
    let shstrtab = &elf.data[shstrtab_off..shstrtab_off + shstrtab_sz];

    sections
        .iter()
        .map(|sh| {
            let name_off = sh.sh_name(endian) as usize;
            let name = read_cstr(shstrtab, name_off)
                .unwrap_or("<invalid>")
                .to_owned();

            Ok(SectionInfo {
                name,
                vma: sh.sh_addr(endian),
                offset: sh.sh_offset(endian),
                size: sh.sh_size(endian),
                sh_type: sh.sh_type(endian),
                sh_flags: sh.sh_flags(endian),
            })
        })
        .collect()
}

/// A single ELF64 program header (segment).
#[derive(Debug, Clone)]
pub struct SegmentInfo {
    /// Segment type (`p_type`), e.g. `PT_LOAD`.
    pub seg_type: u32,
    /// Virtual memory address (`p_addr`).
    pub vma: u64,
    /// File offset (`p_offset`).
    pub offset: u64,
    /// Size in the file (`p_filesz`).
    pub filesz: u64,
    /// Size in memory (`p_memsz`).
    pub memsz: u64,
    /// Segment flags (`p_flags`), e.g. `PF_R | PF_X`.
    pub flags: u32,
}

impl SegmentInfo {
    /// Returns true if the segment is executable (PF_X).
    pub fn is_executable(&self) -> bool {
        self.flags & PF_X != 0
    }

    /// Returns true if the segment is writable (PF_W).
    pub fn is_writable(&self) -> bool {
        self.flags & PF_W != 0
    }
}

impl std::fmt::Display for SegmentInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "type={:#x} vma={:#x} offset={:#x} filesz={} memsz={} flags={:#x}",
            self.seg_type, self.vma, self.offset, self.filesz, self.memsz, self.flags
        )
    }
}

/// Returns all segments in `elf`, in program-header-table order.
pub(crate) fn list_segments(elf: &ElfFile) -> Result<Vec<SegmentInfo>> {
    let parsed = ElfFile64::<Endianness>::parse(elf.data.as_slice())?;
    let endian = parsed.endian();

    Ok(parsed
        .elf_program_headers()
        .iter()
        .map(|ph| SegmentInfo {
            seg_type: ph.p_type(endian),
            vma: ph.p_vaddr(endian),
            offset: ph.p_offset(endian),
            filesz: ph.p_filesz(endian),
            memsz: ph.p_memsz(endian),
            flags: ph.p_flags(endian),
        })
        .collect())
}

/// An existing run of uniform bytes large enough to hold injected code.
#[derive(Debug, Clone)]
pub struct ExistingCave {
    /// Virtual memory address of the run, if it falls inside a LOAD segment.
    pub vma: Option<u64>,
    /// File offset of the run.
    pub offset: u64,
    /// Length of the run in bytes.
    pub size: usize,
    /// The repeated byte value (typically `0x00` or `0x90`).
    pub fill: u8,
}

impl std::fmt::Display for ExistingCave {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let vma = self
            .vma
            .map(|v| format!("{v:#x}"))
            .unwrap_or_else(|| "n/a".to_owned());

        write!(
            f,
            "vma={} offset={:#x} size={} fill={:#04x}",
            vma, self.offset, self.size, self.fill
        )
    }
}

impl ExistingCave {
    /// Returns true if this cave falls within an executable segment.
    pub fn is_executable(&self) -> bool {
        self.vma.is_some()
    }
}

/// Scans `elf` for runs of `0x00` or `0x90` that are at least `min_size` bytes long.
///
/// Results are sorted by size descending so the largest caves appear first.
/// Only LOAD segments are considered when resolving VMAs; runs that fall
/// entirely outside any LOAD segment still appear but with `vma = None`.
pub(crate) fn find_caves(elf: &ElfFile, min_size: usize) -> Result<Vec<ExistingCave>> {
    if min_size == 0 {
        return Err(CaverError::InvalidCaveSize);
    }

    let segments = list_segments(elf)?;
    let mut caves: Vec<ExistingCave> = Vec::new();

    for fill in [0x00u8, 0x90u8] {
        let mut i = 0usize;
        let data = &elf.data;

        while i < data.len() {
            if data[i] != fill {
                i += 1;
                continue;
            }

            let start = i;

            while i < data.len() && data[i] == fill {
                i += 1;
            }

            let run_len = i - start;

            if run_len >= min_size {
                let offset = start as u64;
                let vma = resolve_vma(offset, &segments);
                caves.push(ExistingCave {
                    vma,
                    offset,
                    size: run_len,
                    fill,
                });
            }
        }
    }

    // Largest caves first — most useful for picking injection targets.
    caves.sort_by(|a, b| b.size.cmp(&a.size));

    Ok(caves)
}

/// Reads a null-terminated string from `buf` starting at `offset`.
fn read_cstr(buf: &[u8], offset: usize) -> Option<&str> {
    let slice = buf.get(offset..)?;
    let end = slice.iter().position(|&b| b == 0).unwrap_or(slice.len());

    std::str::from_utf8(&slice[..end]).ok()
}

/// Maps a file `offset` to a VMA using the LOAD segments in `segments`.
fn resolve_vma(offset: u64, segments: &[SegmentInfo]) -> Option<u64> {
    segments
        .iter()
        .filter(|s| s.seg_type == PT_LOAD && s.filesz > 0)
        .find(|s| offset >= s.offset && offset < s.offset + s.filesz)
        .map(|s| s.vma + (offset - s.offset))
}
