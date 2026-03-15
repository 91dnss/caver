//! Architecture-specific metadata and helpers.

use crate::error::{CaverError, Result};

pub mod aarch64;
pub mod x86_64;

/// Supported ELF64 architectures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Arch {
    /// x86_64 (AMD64).
    X86_64,
    /// AArch64.
    AArch64,
}

impl Arch {
    /// Resolves an [`Arch`] from the ELF `e_machine` field.
    pub fn from_e_machine(e_machine: u16) -> Result<Self> {
        match e_machine {
            x86_64::EM_X86_64 => Ok(Arch::X86_64),
            aarch64::EM_AARCH64 => Ok(Arch::AArch64),
            _ => Err(CaverError::UnsupportedArch(e_machine)),
        }
    }

    /// Human-readable architecture name.
    pub fn name(self) -> &'static str {
        match self {
            Arch::X86_64 => "x86_64",
            Arch::AArch64 => "aarch64",
        }
    }

    /// Single-byte NOP fill for this architecture.
    pub fn nop_fill(self) -> &'static [u8] {
        match self {
            Arch::X86_64 => &[x86_64::NOP],
            Arch::AArch64 => &aarch64::NOP,
        }
    }
}
