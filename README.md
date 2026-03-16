# caver

[![Crates.io](https://img.shields.io/crates/v/caver)](https://crates.io/crates/caver)
[![docs.rs](https://img.shields.io/docsrs/caver)](https://docs.rs/caver)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

ELF64 code cave injection library for Rust.

`caver` creates executable code caves in ELF64 binaries by **appending a new loadable segment**. This is useful when a binary has little or no natural slack space for patching or instrumentation.

The injected cave is exported as a `STT_FUNC` symbol (for example `caverfn_mycode`), allowing reverse-engineering tools such as Binary Ninja, Ghidra, and IDA Pro to automatically detect it as a function entry point.

The library focuses only on **structural ELF modification** (creating space for new code). Assembly payloads, hooks, and patching are intended to be written inside a disassembler after injection.

## Goals

`caver` is intentionally small in scope. It aims to:

* Create executable code caves in binaries that lack natural slack space
* Append a new executable `PT_LOAD` segment safely
* Expose the cave as a function symbol for easy discovery in disassemblers
* Provide utilities for inspecting ELF layout and locating existing caves

It does **not** attempt to:

* assemble payloads
* generate trampolines
* automatically hook functions
* patch instructions

These tasks are better handled inside reverse-engineering tools.

## Typical Workflow

A common workflow looks like this:

1. Inject a code cave using `caver`
2. Open the patched binary in a disassembler
3. Write assembly in the injected cave
4. Patch jumps or hooks to redirect execution

Example tools used with this workflow include:

* Binary Ninja
* Ghidra
* IDA Pro

## Supported Architectures

* x86_64
* AArch64
* RISC-V 64

All architectures require ELF64 little-endian binaries.

## Install

Add the crate to your project:
```toml
[dependencies]
caver = "0.1"
```

## Usage

### Inject a cave
```rust
use caver::cave::{CaveOptions, FillByte, inject};
use caver::elf::ElfFile;

fn main() -> caver::error::Result {
    let elf = ElfFile::open("./binary")?;

    let opts = CaveOptions::default()
        .size(512)
        .name(".mycode")
        .fill(FillByte::ArchNop)
        .build()?;

    let patched = inject(&elf, &opts)?;
    println!("{}", patched.info());
    patched.write("./binary_patched")?;

    Ok(())
}
```

### Inject multiple caves
```rust
use caver::cave::{CaveOptions, FillByte, inject_many};
use caver::elf::ElfFile;

fn main() -> caver::error::Result {
    let elf = ElfFile::open("./binary")?;

    let patched = inject_many(&elf, &[
        CaveOptions::default().size(512).name(".mycode").build()?,
        CaveOptions::default().size(256).name(".mydata").fill(FillByte::Zero).build()?,
    ])?;

    for info in patched.infos() {
        println!("{info}");
    }

    patched.write("./binary_patched")?;

    Ok(())
}
```

### Inspect an existing binary

You can inspect the layout of an ELF file and search for existing code caves before injecting:
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

    // only care about caves that are in executable regions
    for cave in elf.find_caves(64)? {
        if cave.is_executable() {
            println!("executable cave: {cave}");
        }
    }

    Ok(())
}
```

## Cave Symbols

Each injected cave is exported as a symbol with the prefix:
```
caverfn_<section_name>
```

For example, injecting `.mycode` produces:
```
caverfn_mycode
```

## License

MIT
