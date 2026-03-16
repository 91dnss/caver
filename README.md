# caver

[![Crates.io](https://img.shields.io/crates/v/caver)](https://crates.io/crates/caver)
[![docs.rs](https://img.shields.io/docsrs/caver)](https://docs.rs/caver)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

ELF64 code cave injection library for Rust.

`caver` creates executable code caves in ELF64 binaries by **appending a new loadable segment**. This is useful when a binary has little or no natural slack space for patching or instrumentation.

The injected cave is exported as a symbol, allowing reverse-engineering tools such as Binary Ninja, Ghidra, and IDA Pro to automatically detect it.

The library focuses only on **structural ELF modification** (creating space for new code). Assembly payloads, hooks, and patching are intended to be written inside a disassembler after injection.

## Goals

`caver` is intentionally small in scope. It aims to:

* Create executable code caves in binaries that lack natural slack space
* Append a new executable `PT_LOAD` segment safely
* Expose the cave as a symbol for easy discovery in disassemblers
* Provide utilities for inspecting ELF layout and locating existing caves

It does **not** attempt to:

* assemble payloads
* generate trampolines
* automatically hook functions
* patch instructions

These tasks are better handled inside reverse-engineering tools.

## Supported Architectures

* x86_64
* AArch64
* RISC-V 64

All architectures require ELF64 little-endian binaries.

## Install
```toml
[dependencies]
caver = "0.2"
```

## Usage

### Inject caves

`name` is the ELF section name and must start with `.`. `symbol` is the exported
symbol name visible in disassemblers — if not set, caver derives one automatically
from the section name.
```rust
use caver::cave::{CaveOptions, FillByte, inject, inject_many};
use caver::elf::ElfFile;

fn main() -> caver::error::Result<()> {
    let elf = ElfFile::open("./binary")?;

    // single cave — auto-generated symbol name
    // default fill byte = ArchNop
    // size() is required
    // name() is required
    // symbol() is optional
    let patched = inject(
        &elf,
        &CaveOptions::builder()
            .size(512)
            .name(".mycode")
            .build()?,
    )?;

    println!("{}", patched.info());
    patched.write("./binary_patched")?;

    // multiple caves — mix of auto and custom symbol names
    let patched = inject_many(&elf, &[
        CaveOptions::builder()
            .size(512)
            .name(".mycode")
            .build()?,
        CaveOptions::builder()
            .size(256)
            .name(".mydata")
            .fill(FillByte::Zero) // data cave
            .symbol("my_cool_data")
            .build()?,
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

### Inspect an existing binary
```rust
use caver::elf::ElfFile;

fn main() -> caver::error::Result<()> {
    let elf = ElfFile::open("./binary")?;

    // find executable sections
    for s in elf.sections()? {
        if s.is_executable() {
            println!("executable section: {}", s.name);
        }
    }

    // find writable segments
    for s in elf.segments()? {
        if s.is_writable() {
            println!("writable segment at {:#x}", s.vma);
        }
    }

    // find existing unused regions
    for cave in elf.find_caves(64)? {
        if cave.is_executable() {
            println!("executable cave: {cave}");
        }
    }

    Ok(())
}
```

## Cave Symbols

The exported symbol name is derived automatically from the section name and fill pattern:

| Fill | Auto symbol | Disassembler treatment |
|------|-------------|------------------------|
| `FillByte::ArchNop` | `caverfn_<name>` | function entry point |
| `FillByte::Zero` | `caverobj_<name>` | data variable |

For example, injecting `.mycode` with `FillByte::ArchNop` produces `caverfn_mycode`, which most disassemblers will automatically treat as a function. Injecting `.mydata` with `FillByte::Zero` produces `caverobj_mydata`, treated as a data variable.

Use `.symbol("my_name")` in the builder to override the auto-generated name entirely.

Note that `name` and `symbol` serve different purposes — `name` is the internal ELF section name used to identify the cave structurally, while `symbol` is the exported name visible to disassemblers and tooling.

## Finding Natural Caves

The `find_caves` function scans the binary for runs of `0x00` or `0x90` bytes that meet a minimum size threshold. Results are sorted largest first. This is useful for locating existing slack space before deciding whether to inject a new segment at all.

## License

MIT
