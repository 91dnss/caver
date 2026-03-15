# caver

ELF64 code cave injection library for Rust.

`caver` creates executable code caves in ELF64 binaries by **appending a new loadable segment**. This is useful when a binary has little or no natural slack space for patching or instrumentation.

The injected cave is exported as a `STT_FUNC` symbol (for example `caverfn_mycode`), allowing reverse-engineering tools such as Binary Ninja, Ghidra, and IDA Pro to automatically detect it as a function.

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

Because the cave is exported as a function symbol, most disassemblers will automatically recognize it as a function entry point.

## Install

Add the crate to your project:

```toml
[dependencies]
caver = "0.1"
```

## Usage

### Inspect an existing binary

You can inspect the layout of an ELF file and search for existing code caves.

```rust
use caver::cave::{find_caves, list_sections, list_segments};
use caver::elf::ElfFile;

fn main() -> caver::error::Result<()> {
    let elf = ElfFile::open("./binary")?;

    for s in list_sections(&elf)? {
        println!("{s}");
    }

    for s in list_segments(&elf)? {
        println!("{s}");
    }

    for cave in find_caves(&elf, 64)? {
        if let Some(vma) = cave.vma {
            println!("VMA: {:#x}, size: {}", vma, cave.size);
        }
    }

    Ok(())
}
```

### Inject a new cave

Create a new executable cave by appending a loadable segment.

```rust
use caver::cave::{CaveOptions, FillByte, inject, inject_many};
use caver::elf::ElfFile;

fn main() -> caver::error::Result<()> {
    let elf = ElfFile::open("./binary")?;

    // single cave
    let opts = CaveOptions::new(512, ".mycode", FillByte::Nop)?;
    let (patched, info) = inject(&elf, &opts)?;

    println!("{info}");
    std::fs::write("./binary_patched", &patched)?;

    // multiple caves in one pass
    let (patched, infos) = inject_many(&elf, &[
        CaveOptions::new(512, ".mycode", FillByte::Nop)?,
        CaveOptions::new(256, ".mydata", FillByte::Zero)?,
    ])?;

    for info in &infos {
        println!("{info}");
    }

    std::fs::write("./binary_patched", &patched)?;

    Ok(())
}
```

The returned `info` contains metadata about the injected cave such as its virtual address, size, and generated symbol name.

## Cave Symbols

Each injected cave is exported as a symbol with the prefix:

```
caverfn_<section_name>
```

For example:

```
caverfn_mycode
```

Because the symbol type is `STT_FUNC`, most disassemblers will treat it as a function entry point automatically.

This makes the cave easy to locate when loading the patched binary.

## Finding Natural Caves

Although `caver` primarily exists to create new executable space, it can also locate existing unused regions in a binary.

The `find_caves` function scans segments for unused regions that may already be suitable for patching.

## Supported Format

Currently supported:

* ELF64 binaries

Tested primarily with:

* Linux x86_64

Other ELF64 architectures may work but are not guaranteed.

## License

MIT
