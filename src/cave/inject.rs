//! Code cave construction and LOAD segment injection.

use object::Endianness;
use object::elf::*;
use object::read::elf::{ElfFile64, FileHeader, ProgramHeader, SectionHeader};

use super::binary::{
    PHDR_SIZE, SHDR_SIZE, align_up, build_shdr, build_sym64, make_phdr, serialise_phdr,
    serialise_shdr, write_u16_le, write_u64_le,
};
use super::types::{CaveInfo, CaveOptions};
use crate::elf::ElfFile;
use crate::error::{CaverError, Result};

/// Injects a code cave into `elf` according to `opts`, returning modified bytes and [`CaveInfo`].
pub fn inject(elf: &ElfFile, opts: &CaveOptions) -> Result<(Vec<u8>, CaveInfo)> {
    let parsed = ElfFile64::<Endianness>::parse(elf.data.as_slice())?;
    let endian = parsed.endian();

    // ── Phdr ─────────────────────────────────────────────────────────────────

    let existing_phdrs: Vec<u8> = parsed
        .elf_program_headers()
        .iter()
        .flat_map(|ph| serialise_phdr(ph, endian))
        .collect();
    let phdr_count = parsed.elf_program_headers().len();

    let last_load = parsed
        .elf_program_headers()
        .iter()
        .filter(|ph| ph.p_type(endian) == PT_LOAD)
        .max_by_key(|ph| ph.p_vaddr(endian))
        .ok_or(CaverError::NotElf64)?;

    let align: u64 = 0x1000;
    let cave_vma = align_up(last_load.p_vaddr(endian) + last_load.p_memsz(endian), align);

    // ── Section headers & string tables ──────────────────────────────────────

    let elf_header = parsed.elf_header();
    let shstrndx = elf_header.e_shstrndx(endian) as usize;
    let sections: &[SectionHeader64<Endianness>] =
        elf_header.section_headers(endian, elf.data.as_slice())?;

    let shdr_count = sections.len();

    let symtab_idx = sections
        .iter()
        .position(|s| s.sh_type(endian) == SHT_SYMTAB);

    let strtab_idx = symtab_idx.map(|i| sections[i].sh_link(endian) as usize);

    // ── Build new shstrtab ────────────────────────────────────────────────────

    let old_shstrtab_sh = sections.get(shstrndx).ok_or(CaverError::NotElf64)?;
    let old_shstrtab_off = old_shstrtab_sh.sh_offset(endian) as usize;
    let old_shstrtab_sz = old_shstrtab_sh.sh_size(endian) as usize;
    let old_shstrtab = &elf.data[old_shstrtab_off..old_shstrtab_off + old_shstrtab_sz];

    // The cave section name is appended at the current end of the table
    let cave_shname_offset = old_shstrtab_sz as u32;
    let mut new_shstrtab = old_shstrtab.to_vec();

    new_shstrtab.extend_from_slice(opts.name.as_bytes());
    new_shstrtab.push(0u8);

    // ── Build new strtab ──────────────────────────────────────────────────────

    // Synthesise a symbol name derived from the section name, e.g. `.cave` → `caverfn_cave`
    let sym_name = format!("caverfn_{}", opts.name.trim_start_matches('.'));

    let (new_strtab, sym_name_offset) = if let Some(idx) = strtab_idx {
        let sh = &sections[idx];
        let off = sh.sh_offset(endian) as usize;
        let sz = sh.sh_size(endian) as usize;
        let mut st = elf.data[off..off + sz].to_vec();
        let name_off = st.len() as u32;
        st.extend_from_slice(sym_name.as_bytes());
        st.push(0u8);
        (st, name_off)
    } else {
        // No existing strtab — create a minimal one (leading null + name)
        let mut st = vec![0u8];
        let name_off = 1u32;
        st.extend_from_slice(sym_name.as_bytes());
        st.push(0u8);
        (st, name_off)
    };

    // ── Build new symtab ──────────────────────────────────────────────────────

    // `st_info = (STB_GLOBAL << 4) | STT_FUNC` per the ELF64 ST_INFO macro
    let new_sym = build_sym64(
        sym_name_offset,
        cave_vma,
        opts.size as u64,
        (STB_GLOBAL << 4) | STT_FUNC,
        STV_DEFAULT,
        (shdr_count + 1) as u16, // shndx of the cave section, appended below
    );

    let new_symtab = if let Some(idx) = symtab_idx {
        let sh = &sections[idx];
        let off = sh.sh_offset(endian) as usize;
        let sz = sh.sh_size(endian) as usize;
        let mut st = elf.data[off..off + sz].to_vec();
        st.extend_from_slice(&new_sym);
        st
    } else {
        // No existing symtab — create one with a mandatory null entry followed by the new symbol
        let mut st = vec![0u8; 24];
        st.extend_from_slice(&new_sym);
        st
    };

    // ── Layout ───────────────────────────────────────────────────────────────

    let orig_len = elf.data.len() as u64;

    // All new data is appended after the original image in this order:
    //   [ existing phdrs ] [ new cave phdr ] [ cave bytes ]
    //   [ shstrtab ] [ strtab ] [ symtab ] [ shdr table ]
    let new_phdr_table_offset = orig_len;
    let new_phdr_table_size = (phdr_count as u64 + 1) * PHDR_SIZE;
    let cave_file_offset = new_phdr_table_offset + new_phdr_table_size;
    let cave_size = opts.size as u64;

    let new_shstrtab_offset = cave_file_offset + cave_size;
    let new_strtab_offset = new_shstrtab_offset + new_shstrtab.len() as u64;
    let new_symtab_offset = new_strtab_offset + new_strtab.len() as u64;

    // +1 for the cave section itself; +2 more if we're creating strtab/symtab from scratch
    let extra_shdrs: u64 = 1 + if symtab_idx.is_none() { 2 } else { 0 };
    let new_shdr_table_offset = new_symtab_offset + new_symtab.len() as u64;

    // ── Build Shdr table ──────────────────────────────────────────────────────

    let new_strtab_shndx = strtab_idx.unwrap_or(shdr_count + 1);

    let mut new_shdr_table: Vec<u8> =
        Vec::with_capacity((shdr_count + extra_shdrs as usize) * SHDR_SIZE as usize);

    for (i, sh) in sections.iter().enumerate() {
        if i == shstrndx {
            // Redirect shstrtab to its new location and updated size
            new_shdr_table.extend_from_slice(&build_shdr(
                sh.sh_name(endian),
                sh.sh_type(endian),
                sh.sh_flags(endian) as u64,
                sh.sh_addr(endian) as u64,
                new_shstrtab_offset,
                new_shstrtab.len() as u64,
                sh.sh_link(endian),
                sh.sh_info(endian),
                sh.sh_addralign(endian) as u64,
                sh.sh_entsize(endian) as u64,
            ));
        } else if Some(i) == strtab_idx {
            // Redirect strtab to its new location and updated size
            new_shdr_table.extend_from_slice(&build_shdr(
                sh.sh_name(endian),
                sh.sh_type(endian),
                sh.sh_flags(endian) as u64,
                sh.sh_addr(endian) as u64,
                new_strtab_offset,
                new_strtab.len() as u64,
                sh.sh_link(endian),
                sh.sh_info(endian),
                sh.sh_addralign(endian) as u64,
                sh.sh_entsize(endian) as u64,
            ));
        } else if Some(i) == symtab_idx {
            // Redirect symtab; update sh_link to point at the (possibly new) strtab shndx
            new_shdr_table.extend_from_slice(&build_shdr(
                sh.sh_name(endian),
                SHT_SYMTAB,
                sh.sh_flags(endian) as u64,
                0,
                new_symtab_offset,
                new_symtab.len() as u64,
                new_strtab_shndx as u32,
                sh.sh_info(endian),
                sh.sh_addralign(endian) as u64,
                24,
            ));
        } else {
            new_shdr_table.extend_from_slice(&serialise_shdr(sh, endian));
        }
    }

    // Append section header for the cave itself
    new_shdr_table.extend_from_slice(&build_shdr(
        cave_shname_offset,
        SHT_PROGBITS,
        (SHF_ALLOC | SHF_EXECINSTR) as u64,
        cave_vma,
        cave_file_offset,
        cave_size,
        0,
        0,
        0x10,
        0,
    ));

    // If no symtab existed, append fresh strtab and symtab section headers
    if symtab_idx.is_none() {
        new_shdr_table.extend_from_slice(&build_shdr(
            0,
            SHT_STRTAB,
            0,
            0,
            new_strtab_offset,
            new_strtab.len() as u64,
            0,
            0,
            1,
            0,
        ));

        new_shdr_table.extend_from_slice(&build_shdr(
            0,
            SHT_SYMTAB,
            0,
            0,
            new_symtab_offset,
            new_symtab.len() as u64,
            new_strtab_shndx as u32,
            1,
            8,
            24,
        ));
    }

    // ── Assemble output ───────────────────────────────────────────────────────

    let mut out = elf.data.clone();

    out.extend_from_slice(&existing_phdrs);
    out.extend_from_slice(&make_phdr(
        PT_LOAD,
        PF_R | PF_X,
        cave_file_offset,
        cave_vma,
        cave_vma, // p_paddr == p_vaddr; physical addressing unused on Linux x86-64
        cave_size,
        cave_size, // p_memsz == p_filesz; no BSS tail
        align,
    ));

    out.extend(std::iter::repeat(opts.fill.value()).take(opts.size));
    out.extend_from_slice(&new_shstrtab);
    out.extend_from_slice(&new_strtab);
    out.extend_from_slice(&new_symtab);
    out.extend_from_slice(&new_shdr_table);

    // ── Patch ELF header ──────────────────────────────────────────────────────

    // e_phoff  (offset 32): new phdr table is appended right after the original image
    // e_shoff  (offset 40): shdr table is at the very end
    // e_phnum  (offset 56): one extra phdr for the cave LOAD segment
    // e_shnum  (offset 60): original count plus the extra section(s) we appended
    write_u64_le(&mut out, 32, new_phdr_table_offset);
    write_u64_le(&mut out, 40, new_shdr_table_offset);
    write_u16_le(&mut out, 56, (phdr_count + 1) as u16);
    write_u16_le(&mut out, 60, (shdr_count as u64 + extra_shdrs) as u16);

    let info = CaveInfo {
        vma: cave_vma,
        offset: cave_file_offset,
        size: opts.size,
        name: opts.name.clone(),
    };

    Ok((out, info))
}

/// Injects multiple code caves in sequence, each built on top of the previous result.
///
/// Caves are applied in `opts` order. On the first error the function returns
/// immediately; successfully injected caves up to that point are discarded.
pub fn inject_many(elf: &ElfFile, opts: &[CaveOptions]) -> Result<(Vec<u8>, Vec<CaveInfo>)> {
    if opts.is_empty() {
        return Ok((elf.data.clone(), vec![]));
    }

    let mut infos = Vec::with_capacity(opts.len());
    let (mut bytes, info) = inject(elf, &opts[0])?;

    infos.push(info);

    for opt in &opts[1..] {
        let next_elf = ElfFile::from_bytes(bytes)?;
        let (new_bytes, info) = inject(&next_elf, opt)?;

        bytes = new_bytes;
        infos.push(info);
    }

    Ok((bytes, infos))
}
