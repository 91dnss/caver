use caver::cave::{CaveOptions, FillByte, inject_many};
use caver::elf::ElfFile;

fn main() -> caver::error::Result<()> {
    let elf = ElfFile::open("/home/dns/Desktop/main")?;

    let (patched, infos) = inject_many(
        &elf,
        &[
            CaveOptions::new(1024, ".cave1", FillByte::Nop)?,
            CaveOptions::new(512, ".cave2", FillByte::Zero)?,
        ],
    )?;

    for info in &infos {
        println!("{info}");
    }

    std::fs::write("/home/dns/Desktop/main_patched", &patched)?;

    Ok(())
}
