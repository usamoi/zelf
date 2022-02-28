use flate2::read::GzDecoder;
use std::io::Read;
use std::ops::{BitAnd, Deref};
use zelf::context::Context;
use zelf::elf::Variant;
use zelf::program::{Program, Programs};
use zelf::section::{Section, SectionFlags32, Sections, Shstrtab};

pub fn decompress<'a, T: Context>(section: Section<'a, T>) -> impl Deref<Target = [u8]> + 'a
where
    <T as zelf::context::Context>::Integer: BitAnd<Output = T::Integer>,
{
    enum Return<'a> {
        Borrowed(&'a [u8]),
        Owned(Vec<u8>),
    }
    impl Deref for Return<'_> {
        type Target = [u8];

        fn deref(&self) -> &Self::Target {
            use Return::*;
            match self {
                Borrowed(r) => *r,
                Owned(v) => v,
            }
        }
    }
    use zelf::compression::Compression;
    use zelf::compression::CompressionType::*;
    let empty = Into::<T::Integer>::into(0);
    let compressed = Into::<T::Integer>::into(SectionFlags32::COMPRESSED.0);
    let flags = section.header().flags().into();
    if flags & compressed != empty {
        let compression = Compression::<T>::parse(section.content()).unwrap();
        match compression.header().typa() {
            Zlib => Return::Owned(
                GzDecoder::new(compression.content())
                    .bytes()
                    .map(Result::unwrap)
                    .collect(),
            ),
            _ => panic!("Unknown compression algorithm."),
        }
    } else {
        Return::Borrowed(section.content())
    }
}

fn format(s: &str, width: usize) -> String {
    let mut s = String::from(s);
    if s.len() <= width {
        while s.len() < width {
            s.push(' ');
        }
    } else {
        while s.len() > width - 5 {
            s.pop();
        }
        s.push_str("[...]");
    }
    s
}

#[rustfmt::skip]
pub fn show<'a, T: Context>(elf: Variant<'a, T>)
where
    <T as zelf::context::Context>::Integer: BitAnd<Output = T::Integer>,
    <T as zelf::context::Context>::Integer: std::fmt::LowerHex,
    <T as zelf::context::Context>::SectionFlags: std::fmt::LowerHex,
{
    use zelf::{Class::*, Data::*, Version::*};
    println!("ELF Header:");
    println!("  Magic:   {:?}", elf.header().ident().as_bytes());
    println!("  Class:                             {}", match T::CLASS { Class32 => "ELF32", Class64 => "ELF64" });
    println!("  Data:                              {}", match T::DATA { Little => "2's complement, little endian", Big => "2's complement, big endian" });
    println!("  Version:                           {}", match T::VERSION { One => "1 (current)" });
    println!("  OS/ABI:                            {:?}", elf.header().ident().os_abi());
    println!("  ABI Version:                       {:?}", elf.header().ident().abi_version());
    println!("  Type:                              {:?}", elf.header().typa());
    println!("  Machine:                           {:#x}", elf.header().machine());
    println!("  Verison:                           {:#x}", elf.header().version());
    println!("  Entry point address:               {:#x}", elf.header().entry());
    println!("  Start of program headers:          {:#?} (bytes into file)", elf.header().phoff());
    println!("  Start of section headers:          {:#?} (bytes into file)", elf.header().shoff());
    println!("  Flags:                             {:#x}", elf.header().flags());
    println!("  Size of this header:               {:?} (bytes)", elf.header().shentsize());
    println!("  Size of program headers:           {:?} (bytes)", elf.header().phentsize());
    println!("  Number of program headers:         {:?}", elf.header().phnum());
    println!("  Size of section headers:           {:?} (bytes)", elf.header().shentsize());
    println!("  Number of section headers:         {:?} (bytes)", elf.header().shnum());
    println!("  Section header string table index: {:?}", elf.header().shstrndx());
    println!();
    if let Some(sections) = Sections::parse(elf).unwrap() {
        let shstrtab = Shstrtab::parse(sections).unwrap().unwrap();
        println!("Section Headers:");
        println!("  [No]  Name              Type              Address           Align");
        println!("        Size              EntSize           Flags  Link  Info");
        for i in 0..sections.num() {
            use zelf::section::SectionType::*;
            let section = Section::parse(sections, i).unwrap().unwrap();
            let name = shstrtab
                .strtab()
                .find(section.header().name() as usize)
                .map(core::str::from_utf8)
                .unwrap_or(Ok("<Lost>"))
                .unwrap_or("<Invaild UTF-8 String>");
            let typa = format!("{:?}", section.header().typa());
            print!("  [{:2}]", i);
            print!("  {}", format(name, 16));
            print!("  {}", format(&typa, 16));
            print!("  {:016x}", section.header().addr());
            print!("  {:016x}", section.header().addralign());
            println!();
            print!("      ");
            print!("  {:016x}", section.header().size());
            print!("  {:016x}", section.header().entsize());
            print!("  {:4x}", section.header().flags());
            print!("  {:4x}", section.header().link());
            print!("  {:4x}", section.header().info());
            println!();
            match section.header().typa() {
                Symtab | Dynsym => {
                    zelf::symtab::Symtab::<T>::parse(&decompress(section)).unwrap();
                }
                Strtab => {
                    zelf::strtab::Strtab::parse(&decompress(section)).unwrap();
                }
                Rela => {
                    zelf::rela::Rela::<T>::parse(&decompress(section)).unwrap();
                }
                Hash => {
                    zelf::hash::Hash::<T>::parse(&decompress(section)).unwrap();
                }
                Dynamic => {
                    zelf::dynamic::Dynamic::<T>::parse(&decompress(section)).unwrap();
                }
                Note => {
                    let content = decompress(section);
                    let note = zelf::note::Note::parse::<T>(&content).unwrap();
                    let name =
                        core::str::from_utf8(note.name()).unwrap_or("<Invaild UTF-8 String>");
                    println!("    [note: {}, {:?}]", name, note.descriptor());
                }
                Rel => {
                    zelf::rel::Rel::<T>::parse(&decompress(section)).unwrap();
                }
                InitArray | FiniArray | PreinitArray => {
                    zelf::array::Array::<T>::parse(&decompress(section)).unwrap();
                }
                Group => {
                    zelf::group::Group::<T>::parse(&decompress(section)).unwrap();
                }
                SymtabShndx => {
                    zelf::shndx::Shndx::<T>::parse(&decompress(section)).unwrap();
                }
                _ => (),
            }
        }
        println!();
    }
    if let Some(programs) = Programs::parse(elf).unwrap() {
        println!("Program Headers:");
        println!("  Type        VirtAddr          PhysAddr          Align");
        println!("              FileSiz           MemSiz            Flags");
        for i in 0..programs.num() {
            use zelf::program::ProgramType::*;
            let program = Program::parse(programs, i).unwrap().unwrap();
            let typa = format!("{:?}", program.header().typa());
            print!("  {}", format(&typa, 10));
            print!("  {:016x}", program.header().vaddr());
            print!("  {:016x}", program.header().paddr());
            print!("  {:016x}", program.header().align());
            println!();
            print!("            ");
            print!("  {:016x}", program.header().filesz());
            print!("  {:016x}", program.header().memsz());
            print!("  {:016x}", program.header().flags());
            println!();
            match program.header().typa() {
                Interp => {
                    let interp = zelf::interp::Interp::parse(program.content()).unwrap();
                    let path = core::str::from_utf8(interp.path()).unwrap();
                    println!("    [requesting program interpeter: {}]", path);
                }
                Dynamic => {
                    zelf::dynamic::Dynamic::<T>::parse(program.content()).unwrap();
                }
                Note => {
                    let note = zelf::note::Note::parse::<T>(program.content()).unwrap();
                    let name =
                        core::str::from_utf8(note.name()).unwrap_or("<Invaild UTF-8 String>");
                    println!("    [note: {}, {:?}]", name, note.descriptor());
                }
                _ => (),
            }
        }
        println!();
    }
}
