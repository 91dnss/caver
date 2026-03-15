//! x86_64-specific constants.

/// x86_64 architecture identifier in the ELF header.
pub const EM_X86_64: u16 = 0x3E;

/// Single-byte x86_64 NOP instruction.
pub const NOP: u8 = 0x90;
