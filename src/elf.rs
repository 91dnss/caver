//! ELF64 parsing and validation helpers.

use crate::arch::Arch;
use crate::error::{CaverError, Result};
use object::{Object, ObjectKind};
use std::path::Path;

/// A parsed and validated ELF64 little endian binary.
pub struct ElfFile {
    /// Raw bytes of the binary.
    pub data: Vec<u8>,
}

impl ElfFile {
    /// Reads a file from `path`, validates it is Elf64, LE, and returns an [`ElfFile`].
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let data = std::fs::read(path)?;
        validate_elf64(&data)?;

        Ok(Self { data })
    }

    /// Validates `bytes` and wraps them in an [`ElfFile`] without touching the filesystem.
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self> {
        validate_elf64(&bytes)?;

        Ok(Self { data: bytes })
    }

    /// Returns the parsed ELF architecture.
    pub fn arch(&self) -> Result<Arch> {
        if self.data.len() < 0x14 {
            return Err(CaverError::NotElf64);
        }

        let e_machine = u16::from_le_bytes([self.data[0x12], self.data[0x13]]);
        Arch::from_e_machine(e_machine)
    }

    /// Returns a parsed [`object::File`] view over raw bytes.
    pub fn parsed(&self) -> Result<object::File<'_>> {
        Ok(object::File::parse(self.data.as_slice())?)
    }
}

/// Validates tha `data` is an Elf64 little-endian binary.
fn validate_elf64(data: &[u8]) -> Result<()> {
    // ELF magic: 0x7f 'E' 'L' 'F'
    if data.len() < 16 || &data[0..4] != b"\x7fELF" {
        return Err(CaverError::NotElf64);
    }

    // EI_CLASS == 2 (64-bit), EI_DATA == 1 (little-endian)
    if data[4] != 2 || data[5] != 1 {
        return Err(CaverError::NotElf64);
    }

    // e_machine at offset 0x12 — must be a supported architecture
    let e_machine = u16::from_le_bytes([data[0x12], data[0x13]]);
    Arch::from_e_machine(e_machine)?;

    // Double check via object crate
    let parsed = object::File::parse(data)?;

    if parsed.kind() == ObjectKind::Unknown {
        return Err(CaverError::NotElf64);
    }

    Ok(())
}
