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
