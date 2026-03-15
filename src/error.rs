//! Error types for caver operations.

use thiserror::Error;

/// All errors that caver can produce.
#[derive(Debug, Error)]
pub enum CaverError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("not an ELF64 little-endian binary")]
    NotElf64,

    #[error("object parse error: {0}")]
    Parse(#[from] object::Error),

    #[error("invalid cave size: must be > 0")]
    InvalidCaveSize,

    #[error("cave name must start with '.'")]
    InvalidCaveName,

    #[error("unsupported architecture (e_machine = {0:#x})")]
    UnsupportedArch(u16),
}

/// Convenience alias used throughout the crate.
pub type Result<T> = std::result::Result<T, CaverError>;
