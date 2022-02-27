extern crate zelf;

use zelf::context::Context;
use zelf::elf::Elf;
use zelf::program::{Program, Programs};
use zelf::section::{Section, Sections, Shstrtab};

pub fn show<'a, T: Context>(elf: Elf<'a, T>)
where
    <T as zelf::context::Context>::Integer: std::fmt::LowerHex,
    <T as zelf::context::Context>::SectionFlags: std::fmt::LowerHex,
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
    if let Some(sections) = Sections::parse(elf).unwrap() {
        let shstrtab = Shstrtab::parse(sections).unwrap().unwrap();
        println!("Section Headers:");
        for i in 0..sections.num() {
            use zelf::section::SectionType::*;
            let section = Section::parse(sections, i).unwrap().unwrap();
            let name = shstrtab
                .strtab()
                .find(section.header().name() as usize)
                .map(core::str::from_utf8)
                .unwrap_or(Ok("<Lost>"))
                .unwrap_or("<Invaild UTF-8 String>");
            print!("  ");
            print!("[{}] ", i);
            print!("name = {}; ", name);
            print!("type = {:?}; ", section.header().typa());
            print!("addr = {:#x}; ", section.header().addr());
            print!("align = {:#x}; ", section.header().addralign());
            print!("flags = {:#x}; ", section.header().flags());
            print!("info = {:#x}; ", section.header().info());
            print!("link = {:#x}; ", section.header().link());
            println!();
            match section.header().typa() {
                Symtab | Dynsym => {
                    zelf::symtab::Symtab::<T>::parse(section.content()).unwrap();
                }
                Strtab => {
                    zelf::strtab::Strtab::parse(section.content()).unwrap();
                }
                Rela => {
                    zelf::rela::Rela::<T>::parse(section.content()).unwrap();
                }
                Hash => {
                    zelf::hash::Hash::<T>::parse(section.content()).unwrap();
                }
                Dynamic => {
                    zelf::dynamic::Dynamic::<T>::parse(section.content()).unwrap();
                }
                Note => {
                    print!("  ");
                    print!("  ");
                    let note = zelf::note::Note::parse::<T>(section.content()).unwrap();
                    let name =
                        core::str::from_utf8(note.name()).unwrap_or("<Invaild UTF-8 String>");
                    print!("name = {}, descriptor = {:?}", name, note.descriptor());
                    println!();
                }
                Rel => {
                    zelf::rel::Rel::<T>::parse(section.content()).unwrap();
                }
                InitArray | FiniArray | PreinitArray => {
                    zelf::array::Array::<T>::parse(section.content()).unwrap();
                }
                Group => {
                    zelf::group::Group::<T>::parse(section.content()).unwrap();
                }
                SymtabShndx => {
                    zelf::shndx::Shndx::<T>::parse(section.content()).unwrap();
                }
                _ => (),
            }
        }
    }
    if let Some(programs) = Programs::parse(elf).unwrap() {
        println!("Program Headers:");
        for i in 0..programs.num() {
            use zelf::program::ProgramType::*;
            let program = Program::parse(programs, i).unwrap().unwrap();
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
                Dynamic => {
                    zelf::dynamic::Dynamic::<T>::parse(program.content()).unwrap();
                }
                Note => {
                    print!("  ");
                    print!("  ");
                    let note = zelf::note::Note::parse::<T>(program.content()).unwrap();
                    let name =
                        core::str::from_utf8(note.name()).unwrap_or("<Invaild UTF-8 String>");
                    print!("name = {}, descriptor = {:?}", name, note.descriptor());
                    println!();
                }
                _ => (),
            }
        }
    }
}
