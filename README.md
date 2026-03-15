# caver

ELF64 code cave injection library. Appends a new executable segment to a binary.

## Install

```toml
[dependencies]
caver = "0.1"
```

The cave is registered as a `STT_FUNC` symbol (e.g. `caverfn_mycode`) so disassemblers like Binary Ninja auto-detect it as a function.

## Usage

### Inspect an existing binary

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

    for info in &infos { println!("{info}"); }
    std::fs::write("./binary_patched", &patched)?;

    Ok(())
}
```

## License

MIT
