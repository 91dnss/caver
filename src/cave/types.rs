//! Public types for code cave configuration and metadata.

use crate::error::{CaverError, Result};

/// Fill pattern used to populate the code cave.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FillByte {
    /// 0x90 — x86 NOP instruction.
    Nop,
    /// 0x00 — null byte.
    Zero,
}

impl FillByte {
    /// Returns the raw byte value for this fill pattern.
    pub fn value(self) -> u8 {
        match self {
            FillByte::Nop => 0x90,
            FillByte::Zero => 0x00,
        }
    }
}

/// Configuration for a code cave injection.
#[derive(Debug, Clone)]
pub struct CaveOptions {
    /// Number of fill bytes to inject.
    pub size: usize,
    /// ELF section name for the cave (must start with '.').
    pub name: String,
    /// Byte pattern to fill the cave with.
    pub fill: FillByte,
}

impl CaveOptions {
    /// Creates a new [`CaveOptions`], validating size and name constraints.
    pub fn new(size: usize, name: impl Into<String>, fill: FillByte) -> Result<Self> {
        if size == 0 {
            return Err(CaverError::InvalidCaveSize);
        }

        let name = name.into();
        if !name.starts_with('.') {
            return Err(CaverError::InvalidCaveName);
        }

        Ok(Self { size, name, fill })
    }
}

/// Metadata describing an injected cave.
#[derive(Debug, Clone)]
pub struct CaveInfo {
    /// Virtual memory address of the cave.
    pub vma: u64,
    /// File offset of the cave.
    pub offset: u64,
    /// Size of the cave in bytes.
    pub size: usize,
    /// Name of the cave section.
    pub name: String,
}

impl std::fmt::Display for CaveInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} vma={:#x} offset={:#x} size={}",
            self.name, self.vma, self.offset, self.size
        )
    }
}
