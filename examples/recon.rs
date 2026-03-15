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
