# caver

ELF64 code cave injection library. Appends a new executable segment to a binary.

## Install

```toml
[dependencies]
caver = "0.1"
```

The cave is registered as a `STT_FUNC` symbol (e.g. `caverfn_mycode`) so disassemblers like Binary Ninja auto-detect it as a function.

```rust
use caver::cave::{CaveOptions, FillByte, inject, inject_many};
use caver::elf::ElfFile;
 
fn main() -> caver::error::Result<()> {
    // open from file or from bytes
    let elf = ElfFile::open("./binary")?;
    // let elf = ElfFile::from_bytes(std::fs::read("./binary")?)?;
 
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
