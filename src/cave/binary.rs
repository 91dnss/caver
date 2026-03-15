//! Low-level ELF64 serialisation and binary patching helpers.
//!
//! All builders return owned `Vec<u8>` with the exact capacity pre-allocated.
//! Byte order is always little-endian (current support only).

use crate::error::CaverError;
use object::Endianness;
use object::elf::{FileHeader64, ProgramHeader64, SHN_XINDEX, SectionHeader64};
use object::read::elf::{FileHeader, ProgramHeader, SectionHeader};

/// Size in bytes of an ELF64 program header entry.
pub const PHDR_SIZE: u64 = 56;
/// Size in bytes of an ELF64 section header entry.
pub const SHDR_SIZE: u64 = 64;
/// Size in bytes of an ELF64 symbol table entry.
pub const SYM_SIZE: u64 = 24;

/// Rounds `val` up to the nearest multiple of `align`.
///
/// `align` must be a power of two.
pub fn align_up(val: u64, align: u64) -> u64 {
    (val + align - 1) & !(align - 1)
}

/// Serialises a parsed [`ProgramHeader64`] back to [`PHDR_SIZE`] raw little-endian bytes.
pub fn serialise_phdr(ph: &ProgramHeader64<Endianness>, e: Endianness) -> Vec<u8> {
    make_phdr(
        ph.p_type(e),
        ph.p_flags(e),
        ph.p_offset(e),
        ph.p_vaddr(e),
        ph.p_paddr(e),
        ph.p_filesz(e),
        ph.p_memsz(e),
        ph.p_align(e),
    )
}

/// Builds a raw [`PHDR_SIZE`]-byte ELF64 program header.
///
/// `p_paddr` is set equal to `p_vaddr` for the injected LOAD segment since
/// physical addressing is not meaningful on Linux x86-64.
pub fn make_phdr(
    p_type: u32,
    p_flags: u32,
    p_offset: u64,
    p_vaddr: u64,
    p_paddr: u64,
    p_filesz: u64,
    p_memsz: u64,
    p_align: u64,
) -> Vec<u8> {
    let mut b = Vec::with_capacity(PHDR_SIZE as usize);

    b.extend_from_slice(&p_type.to_le_bytes());
    b.extend_from_slice(&p_flags.to_le_bytes());
    b.extend_from_slice(&p_offset.to_le_bytes());
    b.extend_from_slice(&p_vaddr.to_le_bytes());
    b.extend_from_slice(&p_paddr.to_le_bytes());
    b.extend_from_slice(&p_filesz.to_le_bytes());
    b.extend_from_slice(&p_memsz.to_le_bytes());
    b.extend_from_slice(&p_align.to_le_bytes());

    b
}

/// Serialises a parsed [`SectionHeader64`] back to [`SHDR_SIZE`] raw little-endian bytes.
pub fn serialise_shdr(sh: &SectionHeader64<Endianness>, e: Endianness) -> Vec<u8> {
    build_shdr(
        sh.sh_name(e),
        sh.sh_type(e),
        sh.sh_flags(e) as u64,
        sh.sh_addr(e) as u64,
        sh.sh_offset(e),
        sh.sh_size(e),
        sh.sh_link(e),
        sh.sh_info(e),
        sh.sh_addralign(e) as u64,
        sh.sh_entsize(e) as u64,
    )
}

/// Builds a raw [`SHDR_SIZE`]-byte ELF64 section header from its fields.
#[allow(clippy::too_many_arguments)]
pub fn build_shdr(
    sh_name: u32,
    sh_type: u32,
    sh_flags: u64,
    sh_addr: u64,
    sh_offset: u64,
    sh_size: u64,
    sh_link: u32,
    sh_info: u32,
    sh_addralign: u64,
    sh_entsize: u64,
) -> Vec<u8> {
    let mut b = Vec::with_capacity(SHDR_SIZE as usize);

    b.extend_from_slice(&sh_name.to_le_bytes());
    b.extend_from_slice(&sh_type.to_le_bytes());
    b.extend_from_slice(&sh_flags.to_le_bytes());
    b.extend_from_slice(&sh_addr.to_le_bytes());
    b.extend_from_slice(&sh_offset.to_le_bytes());
    b.extend_from_slice(&sh_size.to_le_bytes());
    b.extend_from_slice(&sh_link.to_le_bytes());
    b.extend_from_slice(&sh_info.to_le_bytes());
    b.extend_from_slice(&sh_addralign.to_le_bytes());
    b.extend_from_slice(&sh_entsize.to_le_bytes());

    b
}

/// Builds a raw [`SYM_SIZE`]-byte ELF64 symbol table entry (`Elf64_Sym`).
///
/// `st_info` should be constructed via the standard ELF macro:
/// `(bind << 4) | type`, e.g. `(STB_GLOBAL << 4) | STT_FUNC`.
pub fn build_sym64(
    st_name: u32,
    st_value: u64,
    st_size: u64,
    st_info: u8,
    st_other: u8,
    st_shndx: u16,
) -> Vec<u8> {
    let mut b = Vec::with_capacity(SYM_SIZE as usize);

    b.extend_from_slice(&st_name.to_le_bytes());
    b.push(st_info);
    b.push(st_other);
    b.extend_from_slice(&st_shndx.to_le_bytes());
    b.extend_from_slice(&st_value.to_le_bytes());
    b.extend_from_slice(&st_size.to_le_bytes());

    b
}

pub fn resolve_shstrndx(
    endian: Endianness,
    elf_header: &FileHeader64<Endianness>,
    sections: &[SectionHeader64<Endianness>],
) -> Result<usize, CaverError> {
    let raw = elf_header.e_shstrndx(endian);

    if raw != SHN_XINDEX as u16 {
        return Ok(raw as usize);
    }

    // Real index is in sh_link of section 0
    sections
        .first()
        .map(|s| s.sh_link(endian) as usize)
        .ok_or(CaverError::NotElf64)
}

/// Writes a `u64` little-endian at `offset` in `data`.
pub fn write_u64_le(data: &mut [u8], offset: usize, val: u64) {
    data[offset..offset + 8].copy_from_slice(&val.to_le_bytes());
}

/// Writes a `u16` little-endian at `offset` in `data`.
pub fn write_u16_le(data: &mut [u8], offset: usize, val: u16) {
    data[offset..offset + 2].copy_from_slice(&val.to_le_bytes());
}
