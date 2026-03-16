# caver

[![Crates.io](https://img.shields.io/crates/v/caver)](https://crates.io/crates/caver)
[![docs.rs](https://img.shields.io/docsrs/caver)](https://docs.rs/caver)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

ELF64 code cave injection library for Rust.

`caver` creates code caves in ELF64 binaries by appending a new loadable segment.
It handles only **structural ELF modification** — creating and labelling the space.
Assembly, trampolines, and hooking are left to your disassembler.

Supports x86_64, AArch64, and RISC-V 64 (little-endian ELF64 only).

## Install
```toml
[dependencies]
caver = "0.3"
```

## Cave types

Every cave has a **fill pattern** that controls both its contents and how
disassemblers interpret it:

| Fill | Contents | ELF symbol type | Disassembler sees |
|------|----------|-----------------|-------------------|
| `FillByte::ArchNop` | arch NOP sled | `STT_FUNC` | function entry point |
| `FillByte::Zero` | null bytes | `STT_OBJECT` | data variable |

Use `ArchNop` when you plan to write code into the cave. Use `Zero` when you
need a writable data region.

## Naming

Each cave gets two names that serve different purposes:

- **`name`** — the ELF section name (e.g. `.mycode`). Must start with `.`.
  This is what `readelf`, `objdump`, and `elf.sections()` see.
- **`symbol`** — the exported symbol name (e.g. `caverfn_mycode`). This is
  what Binary Ninja, Ghidra, and IDA see when they auto-analyse the binary.

If you don't call `.symbol()`, caver derives one from the section name:
`caverfn_<name>` for NOP caves, `caverobj_<name>` for zero caves.
```
.mycode  →  caverfn_mycode   (ArchNop, auto)
.mydata  →  caverobj_mydata  (Zero, auto)
.hook    →  my_hook          (ArchNop, overridden)
```

## Usage

### Inject a single cave
```rust
use caver::cave::{CaveOptions, FillByte, inject};
use caver::elf::ElfFile;

fn main() -> caver::error::Result<()> {
    let elf = ElfFile::open("./binary")?;

    let patched = inject(
        &elf,
        &CaveOptions::builder()
            .size(512)
            .name(".mycode")
            // .symbol("my_name")  // optional override
            // .fill(FillByte::Zero)  // default is ArchNop
            .build()?,
    )?;

    // info() returns vma, offset, size, section name, and resolved symbol
    println!("{}", patched.info());
    patched.write("./binary_patched")?;

    Ok(())
}
```

### Inject multiple caves
```rust
use caver::cave::{CaveOptions, FillByte, inject_many};
use caver::elf::ElfFile;

fn main() -> caver::error::Result<()> {
    let elf = ElfFile::open("./binary")?;

    let patched = inject_many(&elf, &[
        // NOP sled — auto symbol: caverfn_mycode
        CaveOptions::builder()
            .size(512)
            .name(".mycode")
            .build()?,
        // Data region — auto symbol: caverobj_mydata
        CaveOptions::builder()
            .size(256)
            .name(".mydata")
            .fill(FillByte::Zero)
            .build()?,
        // NOP sled — custom symbol overrides auto-generation
        CaveOptions::builder()
            .size(128)
            .name(".hook")
            .symbol("my_hook")
            .build()?,
    ])?;

    for info in patched.infos() {
        println!("{info}");
    }

    patched.write("./binary_patched")?;

    Ok(())
}
```

### Inspect a binary
```rust
use caver::elf::ElfFile;

fn main() -> caver::error::Result<()> {
    let elf = ElfFile::open("./binary")?;

    // Check if stripped before expecting symbols
    if elf.is_stripped()? {
        println!("binary is stripped — no .symtab");
    } else {
        for sym in elf.symbols()? {
            println!("{sym}");
        }
    }

    // Sections
    for s in elf.sections()? {
        if s.is_executable() {
            println!("executable section: {}", s.name);
        }
    }

    // Segments
    for s in elf.segments()? {
        if s.is_writable() {
            println!("writable segment at {:#x}", s.vma);
        }
    }

    // Scan for existing slack space before deciding to inject
    for cave in elf.find_caves(64)? {
        if cave.is_executable() {
            println!("existing executable cave: {cave}");
        }
    }

    Ok(())
}
```

## License

MIT
