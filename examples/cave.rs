use caver::cave::{CaveOptions, FillByte, inject_many};
use caver::elf::ElfFile;

fn main() -> caver::error::Result<()> {
    let elf = ElfFile::open("./binary")?;

    let patched = inject_many(
        &elf,
        &[
            CaveOptions::default().size(512).name(".mycode").build()?,
            CaveOptions::default()
                .size(256)
                .name(".mydata")
                .fill(FillByte::Zero)
                .build()?,
        ],
    )?;

    for info in patched.infos() {
        println!("{info}");
    }

    patched.write("./binary_patched")?;

    Ok(())
}
