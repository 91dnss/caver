//! AArch64-specific constants.

/// AArch64 architecture identifier in the ELF header.
pub const EM_AARCH64: u16 = 0xB7;

/// 4-byte AArch64 NOP instruction (little-endian).
pub const NOP: [u8; 4] = [0x1F, 0x20, 0x03, 0xD5];
