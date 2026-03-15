use caver::elf::ElfFile;

fn main() -> caver::error::Result<()> {
    let elf = ElfFile::open("./binary")?;

    for s in elf.sections()? {
        println!("{s}");
    }
    for s in elf.segments()? {
        println!("{s}");
    }
    for cave in elf.find_caves(64)? {
        println!("{cave}");
    }

    Ok(())
}
