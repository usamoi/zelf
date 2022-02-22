extern crate zelf;

use clap::Parser;
use zelf::{elf::Elf, interpret::Interpreter};

#[derive(Parser, Debug)]
#[clap(author, version, about)]
pub struct Args {
    file: Option<String>,
}

fn solve<'a, T: Interpreter>(elf: Elf<'a, T>)
where
    <T as zelf::interpret::Interpreter>::Integer: std::fmt::LowerHex,
{
    println!("ELF Header:");
    println!("  Magic:       {:?}", elf.header().ident().magic);
    println!("  Data:        {:?}", elf.header().ident().data());
    println!("  Class:       {:?}", elf.header().ident().class());
    println!("  Version:     {:?}", elf.header().ident().version());
    println!("  OS ABI:      {:?}", elf.header().ident().os_abi());
    println!("  ABI Version: {:?}", elf.header().ident().abi_version());
    println!("  Object file Type:     {:?}", elf.header().typa());
    println!("  Object file Flags:    {:#x}", elf.header().flags());
    println!("  Object file Verison:  {:#x}", elf.header().version());
    println!("  Object file Machine:  {:#x}", elf.header().machine());
    println!("  Object file Entry:    {:#x}", elf.header().entry());
    if let Some(sections) = elf.sections() {
        println!("Section Headers:");
        for (i, result) in sections.iter().enumerate() {
            let section = result.unwrap();
            let name = elf.shstrtab().map(|strtab| {
                strtab
                    .find(section.header().name() as usize)
                    .map(core::str::from_utf8)
                    .unwrap_or(Ok("<LOST>"))
                    .unwrap_or("<INVAILD>")
            });
            print!("  ");
            print!("[{}] ", i);
            if let Some(name) = name {
                print!("name = {}; ", name);
            }
            print!("type = {:?}; ", section.header().typa());
            print!("addr = {:?}; ", section.header().addr());
            print!("align = {:#x}; ", section.header().addralign());
            // print!("flags = {:#x}; ", section.header().flags());
            print!("info = {:#x}; ", section.header().info());
            print!("link = {:#x}; ", section.header().link());
            println!();
        }
    }
    if let Some(programs) = elf.programs() {
        println!("Program Headers:");
        for (i, result) in programs.iter().enumerate() {
            use zelf::program::ProgramType::*;
            let program = result.unwrap();
            print!("  ");
            print!("[{}] ", i);
            print!("type = {:?}; ", program.header().typa());
            print!("vaddr = {:#x}; ", program.header().vaddr());
            print!("align = {:#x}; ", program.header().align());
            print!("flags = {:#x}; ", program.header().flags());
            print!("paddr = {:#x}; ", program.header().paddr());
            println!();
            match program.header().typa() {
                Interp => {
                    print!("  ");
                    print!("  ");
                    let interp =
                        zelf::interp::Interp::parse(program.content()).expect("Bad .interp");
                    let path = core::str::from_utf8(interp.path()).unwrap();
                    println!("requesting program interpeter: {}", path);
                }
                _ => (),
            }
        }
    }
}

fn main() {
    use zelf::elf::Elfs::*;
    let args = Args::parse();
    let file = args.file.expect("No ELF file is specified.");
    let bytes = std::fs::read(file).expect("Cannot open the file.");
    let elf = zelf::elf::Elfs::parse(&bytes).expect("Failed to parse the file.");
    match elf {
        Little32(elf) => solve(elf),
        Little64(elf) => solve(elf),
        Big32(elf) => solve(elf),
        Big64(elf) => solve(elf),
    }
}