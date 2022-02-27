use std::error::Error;

#[path = "../examples/readelf.rs"]
mod readelf;

pub fn print(file: &str) -> Result<(), Box<dyn Error>> {
    use zelf::elf::Elfs::{self, *};
    use zelf::elf::ParseElfsError::*;
    use zelf::ident::ParseIdentError::*;
    let bytes = std::fs::read(file)?;
    println!("File Name: {}", file);
    let elf = match Elfs::parse(&bytes) {
        Ok(x) => x,
        Err(BadIdent(BadPropertyMagic)) => return Ok(()),
        Err(e) => panic!("{e:?}"),
    };
    match elf {
        Little32(elf) => readelf::display(elf),
        Little64(elf) => readelf::display(elf),
        Big32(elf) => readelf::display(elf),
        Big64(elf) => readelf::display(elf),
    }
    Ok(())
}

#[test]
fn linux() {
    let dir = std::fs::read_dir("/usr/bin").unwrap();
    for each in dir.into_iter() {
        let each = each.unwrap();
        if !each.file_type().unwrap().is_file() {
            continue;
        }
        if let Err(_) = print(format!("/usr/bin/{}", each.file_name().to_str().unwrap()).as_str()) {
        }
    }
}
