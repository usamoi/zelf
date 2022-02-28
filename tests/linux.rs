use std::error::Error;
use zelf::ident::Ident;

#[path = "../utils/show.rs"]
mod show;

fn show(file: &str) -> Result<(), Box<dyn Error>> {
    use zelf::elf::Elf::{self, *};
    use zelf::ident::ParseIdentError::*;
    let bytes = std::fs::read(file)?;
    match Ident::parse(&bytes) {
        Err(BrokenHeader | BadPropertyMagic) => return Ok(()),
        _ => (),
    }
    println!("File Name: {}", file);
    match Elf::parse(&bytes).unwrap() {
        Little32(elf) => show::show(elf),
        Little64(elf) => show::show(elf),
        Big32(elf) => show::show(elf),
        Big64(elf) => show::show(elf),
    }
    Ok(())
}

#[cfg(target_os = "linux")]
#[test]
fn linux() {
    let usr_lib = std::fs::read_dir("/usr/lib").unwrap().into_iter();
    let usr_bin = std::fs::read_dir("/usr/bin").unwrap().into_iter();
    for each in usr_lib.chain(usr_bin) {
        let each = each.unwrap();
        if !each.file_type().unwrap().is_file() {
            continue;
        }
        if let Err(_) = show(&format!("/usr/bin/{}", each.file_name().to_str().unwrap())) {}
    }
}
