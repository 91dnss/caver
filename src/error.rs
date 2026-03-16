//! Error types for caver operations.

use thiserror::Error;

/// All errors that caver can produce.
#[derive(Debug, Error)]
pub enum CaverError {
    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Not an ELF64 little-endian binary.
    #[error("not an ELF64 little-endian binary")]
    NotElf64,

    /// Object parse error.
    #[error("object parse error: {0}")]
    Parse(#[from] object::Error),

    /// Invalid cave size: must be > 0.
    #[error("invalid cave size: must be > 0")]
    InvalidCaveSize,

    /// Cave name must start with '.'.
    #[error("cave name must start with '.'")]
    InvalidCaveName,

    /// Unsupported architecture (e_machine = {0:#x}).
    #[error("unsupported architecture (e_machine = {0:#x})")]
    UnsupportedArch(u16),

    /// Output validation failed: {0}.
    #[error("output validation failed: {0}")]
    ValidationFailed(String),

    /// Section name {0:?} already exists in binary.
    #[error("section name {0:?} already exists in binary")]
    DuplicateSectionName(String),

    /// Cave size {0} is not a multiple of {1} (required for this architecture).
    #[error("cave size {0} is not a multiple of {1} (required for this architecture)")]
    UnalignedCaveSize(usize, usize),

    /// Cave VMA {0:#x} overlaps existing segment at {1:#x}.
    #[error("cave VMA {0:#x} overlaps existing segment at {1:#x}")]
    VmaOverlap(u64, u64),
}

/// Convenience alias used throughout the crate.
pub type Result<T> = std::result::Result<T, CaverError>;
