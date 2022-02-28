extern crate zelf;

#[path = "../utils/show.rs"]
mod show;

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
pub struct Args {
    file: Option<String>,
}

fn show(file: &str) {
    use zelf::elf::Elf::*;
    let bytes = std::fs::read(file).expect("Cannot open the file.");
    let elf = zelf::elf::Elf::parse(&bytes).unwrap();
    match elf {
        Little32(elf) => show::show(elf),
        Little64(elf) => show::show(elf),
        Big32(elf) => show::show(elf),
        Big64(elf) => show::show(elf),
    }
}

fn main() {
    let args = Args::parse();
    let file = args.file.expect("No ELF file is specified.");
    show(&file);
}
