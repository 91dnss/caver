//! RISC-V 64-bit specific constants.

/// RISC-V architecture identifier in the ELF header.
pub const EM_RISCV64: u16 = 0xF3;

/// 4-byte RISC-V NOP instruction (`addi x0, x0, 0`), little-endian.
pub const NOP: [u8; 4] = [0x13, 0x00, 0x00, 0x00];
