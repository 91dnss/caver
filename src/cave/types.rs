//! Public types for code cave configuration and metadata.

use crate::arch::Arch;
use crate::error::{CaverError, Result};

/// Fill pattern used to populate the code cave.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FillByte {
    /// Architecture-specific single-byte NOP instruction.
    ///
    /// Currently resolves to x86_64, but will map to other architectures
    /// as support is added.
    ArchNop,
    /// 0x00 — null byte.
    Zero,
}

impl FillByte {
    /// Returns the raw byte value for this fill pattern.
    pub fn value(self) -> &'static [u8] {
        // Backward-compatible default while only x86_64 is supported
        self.fill_bytes_for(Arch::X86_64)
    }

    /// Returns the raw byte value for this fill pattern for `arch`.
    pub fn fill_bytes_for(self, arch: Arch) -> &'static [u8] {
        match self {
            FillByte::ArchNop => arch.nop_fill(),
            FillByte::Zero => &[0x00],
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
    /// Creates a new [`CaveOptions`] using the builder API.
    pub fn builder() -> CaveOptionsBuilder {
        CaveOptionsBuilder::default()
    }

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

/// Builder for [`CaveOptions`].
#[derive(Debug, Default)]
pub struct CaveOptionsBuilder {
    size: Option<usize>,
    name: Option<String>,
    fill: Option<FillByte>,
}

impl CaveOptionsBuilder {
    pub fn size(mut self, size: usize) -> Self {
        self.size = Some(size);
        self
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn fill(mut self, fill: FillByte) -> Self {
        self.fill = Some(fill);
        self
    }

    pub fn build(self) -> Result<CaveOptions> {
        let size = self.size.ok_or(CaverError::InvalidCaveSize)?;
        let name = self.name.ok_or(CaverError::InvalidCaveName)?;
        let fill = self.fill.unwrap_or(FillByte::ArchNop);

        CaveOptions::new(size, name, fill)
    }
}
