//! Public types for code cave configuration and metadata.

use crate::arch::Arch;
use crate::error::{CaverError, Result};
use object::elf::{STT_FUNC, STT_OBJECT};
use std::path::Path;

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
    /// Returns the raw byte value for this fill pattern for `arch`.
    pub(crate) fn fill_bytes_for(self, arch: Arch) -> &'static [u8] {
        match self {
            FillByte::ArchNop => arch.nop_fill(),
            FillByte::Zero => &[0x00],
        }
    }

    /// Returns the ELF symbol type for this fill pattern.
    pub(crate) fn sym_type(self) -> u8 {
        match self {
            FillByte::ArchNop => STT_FUNC,
            FillByte::Zero => STT_OBJECT,
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
    /// Optional symbol name override.
    pub symbol: Option<String>,
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

        Ok(Self {
            size,
            name,
            fill,
            symbol: None,
        })
    }

    /// Derives the symbol name from options, using override if set.
    pub(crate) fn symbol_name(&self) -> String {
        if let Some(ref s) = self.symbol {
            return s.clone();
        }

        let base = self.name.trim_start_matches('.');

        match self.fill {
            FillByte::ArchNop => format!("caverfn_{base}"),
            FillByte::Zero => format!("caverobj_{base}"),
        }
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
    /// Symbol name of the cave section.
    pub symbol: String,
}

impl std::fmt::Display for CaveInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} symbol={} vma={:#x} offset={:#x} size={}",
            self.name, self.symbol, self.vma, self.offset, self.size
        )
    }
}

/// Builder for [`CaveOptions`].
#[derive(Debug, Default)]
pub struct CaveOptionsBuilder {
    size: Option<usize>,
    name: Option<String>,
    fill: Option<FillByte>,
    symbol: Option<String>,
}

impl CaveOptionsBuilder {
    /// Sets the cave size in bytes. Must be greater than zero.
    pub fn size(mut self, size: usize) -> Self {
        self.size = Some(size);
        self
    }

    /// Sets the ELF section name. Must start with '.'.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Sets the fill pattern. Defaults to [`FillByte::ArchNop`] if not called.
    pub fn fill(mut self, fill: FillByte) -> Self {
        self.fill = Some(fill);
        self
    }

    /// Overrides the auto-generated symbol name.
    ///
    /// If not set, the symbol is derived from the section name:
    /// `caverfn_<name>` for [`FillByte::ArchNop`] and `caverobj_<name>`
    /// for [`FillByte::Zero`].
    pub fn symbol(mut self, symbol: impl Into<String>) -> Self {
        self.symbol = Some(symbol.into());
        self
    }

    /// Validates and builds the [`CaveOptions`].
    pub fn build(self) -> Result<CaveOptions> {
        let size = self.size.ok_or(CaverError::InvalidCaveSize)?;
        let name = self.name.ok_or(CaverError::InvalidCaveName)?;
        let fill = self.fill.unwrap_or(FillByte::ArchNop);
        let mut opts = CaveOptions::new(size, name, fill)?;
        opts.symbol = self.symbol;

        Ok(opts)
    }
}

/// The result of a cave injection operation.
pub struct PatchedElf {
    pub(crate) data: Vec<u8>,
    pub(crate) infos: Vec<CaveInfo>,
}

impl PatchedElf {
    pub(crate) fn new(data: Vec<u8>, infos: Vec<CaveInfo>) -> Self {
        Self { data, infos }
    }

    /// Returns metadata for the single injected cave.
    pub fn info(&self) -> &CaveInfo {
        &self.infos[0]
    }

    /// Returns metadata for all injected caves.
    pub fn infos(&self) -> &[CaveInfo] {
        &self.infos
    }

    /// Writes the patched binary to `path`.
    pub fn write(&self, path: impl AsRef<Path>) -> Result<()> {
        Ok(std::fs::write(path, &self.data)?)
    }
}
