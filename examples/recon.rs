use caver::cave::{find_caves, list_sections, list_segments};
use caver::elf::ElfFile;

fn main() -> caver::error::Result<()> {
    let elf = ElfFile::open("/home/dns/Desktop/main")?;

    for s in list_sections(&elf)? {
        println!("{s}");
    }

    for s in list_segments(&elf)? {
        println!("{s}");
    }

    for cave in find_caves(&elf, 64)? {
        println!("{cave}");
    }

    Ok(())
}
