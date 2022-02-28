use crate::context::PropU32;
use crate::context::*;
use crate::elf::Variant;
use crate::strtab::{ParseStrtabError, Strtab};
use crate::utils::{as_offset, read, read_n, Pod};
use core::marker::PhantomData;
use core::ops::RangeInclusive;

#[derive(Debug, Clone)]
pub enum ParseSectionsError {
    BrokenHeaders,
    BadPropertyShentsize,
    BadPropertyShstrndx,
    BadPropertyShnum,
}

#[derive(Debug, Clone)]
pub enum ParseSectionError {
    BadPropertyType,
    BrokenContent,
}

#[derive(Debug, Clone)]
pub enum ParseShstrtabError {
    BadPropertyShstrndx,
    FromSection(ParseSectionError),
    BadPropertyType,
    FromStrtab(ParseStrtabError),
}

#[derive(Debug, Clone, Copy)]
pub struct Sections<'a, T: Context> {
    data: &'a [u8],
    offset: usize,
    shstrndx: u16,
    num: u16,
    _maker: PhantomData<T>,
}

impl<'a, T: Context> Sections<'a, T> {
    pub fn parse(elf: Variant<'a, T>) -> Result<Option<Sections<'a, T>>, ParseSectionsError> {
        use ParseSectionsError::*;
        let data = elf.data();
        let offset = as_offset::<T>(elf.header().shoff()).ok_or(BrokenHeaders)?;
        if offset == 0 {
            return Ok(None);
        }
        if elf.header().shentsize() as usize != core::mem::size_of::<SectionHeader<T>>() {
            return Err(BadPropertyShentsize);
        }
        // section zero
        let zero = read::<SectionHeader<T>>(data, offset).ok_or(BrokenHeaders)?;
        let num = match (elf.header().shnum(), as_offset::<T>(zero.size())) {
            (x @ 0..=0xfeff, Some(0)) => x,
            (0, Some(x @ 0xff00..=0xffff)) => x as u16,
            _ => return Err(BadPropertyShnum),
        };
        let shstrndx = match (elf.header().shstrndx(), zero.link()) {
            (x @ 0..=0xfeff, 0) => x,
            (SECTION_INDEX_XINDEX, x @ 0xff00..=0xffff) => x as u16,
            _ => return Err(BadPropertyShstrndx),
        };
        read_n::<SectionHeader<T>>(data, offset, num as usize).ok_or(BrokenHeaders)?;
        Ok(Some(Self {
            data,
            offset,
            shstrndx,
            num,
            _maker: PhantomData,
        }))
    }
    pub fn shstrndx(&self) -> u16 {
        self.shstrndx
    }
    pub fn num(&self) -> u16 {
        self.num
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Section<'a, T: Context> {
    sheader: &'a SectionHeader<T>,
    content: &'a [u8],
}

impl<'a, T: Context> Section<'a, T> {
    pub fn parse(
        sections: Sections<'a, T>,
        index: u16,
    ) -> Option<Result<Section<'a, T>, ParseSectionError>> {
        use ParseSectionError::*;
        use SectionType::*;
        if index >= sections.num() {
            return None;
        }
        let offset = sections.offset + index as usize * core::mem::size_of::<SectionHeader<T>>();
        fn helper<'a, T: Context>(
            data: &'a [u8],
            offset: usize,
        ) -> Result<Section<'a, T>, ParseSectionError> {
            let sheader: &'a SectionHeader<T> = read(data, offset).unwrap();
            let typa =
                SectionType::try_from(T::interpret(sheader.typa)).map_err(|_| BadPropertyType)?;
            match typa {
                Null => Ok(Section {
                    sheader,
                    content: &[],
                }),
                Nobits => {
                    let content_offset = as_offset::<T>(sheader.offset()).ok_or(BrokenContent)?;
                    let content = read_n::<u8>(data, content_offset, 0).ok_or(BrokenContent)?;
                    Ok(Section { sheader, content })
                }
                _ => {
                    let content_offset = as_offset::<T>(sheader.offset()).ok_or(BrokenContent)?;
                    let content_size = as_offset::<T>(sheader.size()).ok_or(BrokenContent)?;
                    let content =
                        read_n::<u8>(data, content_offset, content_size).ok_or(BrokenContent)?;
                    Ok(Section { sheader, content })
                }
            }
        }
        Some(helper(sections.data, offset))
    }
    pub fn header(&self) -> &'a SectionHeader<T> {
        self.sheader
    }
    pub fn content(&self) -> &'a [u8] {
        self.content
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Shstrtab<'a> {
    strtab: Strtab<'a>,
}

impl<'a> Shstrtab<'a> {
    pub fn parse<T: Context>(
        sections: Sections<'a, T>,
    ) -> Result<Option<Shstrtab<'a>>, ParseShstrtabError> {
        use ParseShstrtabError::*;
        let shstrndx = match sections.shstrndx {
            SECTION_INDEX_UNDEF => return Ok(None),
            x => x,
        };
        let section = Section::parse(sections, shstrndx)
            .ok_or(BadPropertyShstrndx)?
            .map_err(FromSection)?;
        if section.header().typa() != SectionType::Strtab {
            return Err(BadPropertyType);
        }
        let shstrtab = Strtab::parse(section.content()).map_err(FromStrtab)?;
        Ok(Some(Self { strtab: shstrtab }))
    }
    pub fn strtab(&self) -> Strtab<'a> {
        self.strtab
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct SectionHeader<T: Context> {
    pub name: PropU32,
    pub typa: PropU32,
    pub flags: T::PropUsize,
    pub addr: T::PropUsize,
    pub offset: T::PropUsize,
    pub size: T::PropUsize,
    pub link: PropU32,
    pub info: PropU32,
    pub addralign: T::PropUsize,
    pub entsize: T::PropUsize,
}

impl<T: Context> SectionHeader<T> {
    pub fn name(&self) -> u32 {
        T::interpret(self.name)
    }
    pub fn checked_type(&self) -> Option<SectionType> {
        SectionType::try_from(T::interpret(self.typa)).ok()
    }
    /// # Panics
    ///
    /// Panics if the value is invaild.
    pub fn typa(&self) -> SectionType {
        self.checked_type().unwrap()
    }
    pub fn flags(&self) -> T::SectionFlags {
        From::<T::Integer>::from(T::interpret(self.flags))
    }
    pub fn addr(&self) -> T::Integer {
        T::interpret(self.addr)
    }
    pub fn offset(&self) -> T::Integer {
        T::interpret(self.offset)
    }
    pub fn size(&self) -> T::Integer {
        T::interpret(self.size)
    }
    pub fn link(&self) -> u32 {
        T::interpret(self.link)
    }
    pub fn info(&self) -> u32 {
        T::interpret(self.info)
    }
    pub fn addralign(&self) -> T::Integer {
        T::interpret(self.addralign)
    }
    pub fn entsize(&self) -> T::Integer {
        T::interpret(self.entsize)
    }
}

unsafe impl<T: Context> Pod for SectionHeader<T> {}

/// Undefined value.
pub const SECTION_INDEX_UNDEF: u16 = 0;
/// The range of reserved indexes.
pub const SECTION_INDEX_RESERVE: RangeInclusive<u16> = 0xff00..=0xffff;
/// Processor-specific.
pub const SECTION_INDEX_PROCESSORSPECIFIC: RangeInclusive<u16> = 0xff00..=0xff1f;
/// Operating system-specific.
pub const SECTION_INDEX_OSSPECIFIC: RangeInclusive<u16> = 0xff20..=0xff3f;
/// This value specifies absolute values for the corresponding reference.
pub const SECTION_INDEX_ABS: u16 = 0xfff1;
/// Symbols defined relative to this section are common symbols.
pub const SECTION_INDEX_COMMON: u16 = 0xfff2;
/// This value is an escape value.
/// It indicates that the actual section header index is too large to fit in the containing field and is to be found in another location (specific to the structure where it appears).
pub const SECTION_INDEX_XINDEX: u16 = 0xffff;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SectionType {
    /// Section header table entry unused.
    Null,
    /// Program data.
    Progbits,
    /// Symbol table.
    Symtab,
    /// String table.
    Strtab,
    /// Relocation entries with addends.
    Rela,
    /// Symbol hash table.
    Hash,
    /// Dynamic linking information.
    Dynamic,
    /// Notes.
    Note,
    /// Program space with no data (bss).
    Nobits,
    /// Relocation entries, no addends.
    Rel,
    /// Reserved.
    Shlib,
    /// Dynamic linker symbol table.
    Dynsym,
    /// Array of constructors.
    InitArray,
    /// Array of destructors.
    FiniArray,
    /// Array of pre-constructors.
    PreinitArray,
    /// Section group.
    Group,
    /// Extended section indices.
    SymtabShndx,
    /// Processor-specific.
    OsSpecific(u32),
    /// Operating system-specific.
    ProcessorSpecific(u32),
    /// Reserved for application programs.
    User(u32),
}

impl TryFrom<u32> for SectionType {
    type Error = ();
    fn try_from(value: u32) -> Result<Self, ()> {
        use SectionType::*;
        match value {
            0x0 => Ok(Null),
            0x1 => Ok(Progbits),
            0x2 => Ok(Symtab),
            0x3 => Ok(Strtab),
            0x4 => Ok(Rela),
            0x5 => Ok(Hash),
            0x6 => Ok(Dynamic),
            0x7 => Ok(Note),
            0x8 => Ok(Nobits),
            0x9 => Ok(Rel),
            0xA => Ok(Shlib),
            0xB => Ok(Dynsym),
            0xE => Ok(InitArray),
            0xF => Ok(FiniArray),
            0x10 => Ok(PreinitArray),
            0x11 => Ok(Group),
            0x12 => Ok(SymtabShndx),
            x @ 0x60000000..=0x6fffffff => Ok(OsSpecific(x)),
            x @ 0x70000000..=0x7fffffff => Ok(ProcessorSpecific(x)),
            x @ 0x80000000..=0xffffffff => Ok(User(x)),
            _ => Err(()),
        }
    }
}

impl From<SectionType> for u32 {
    fn from(value: SectionType) -> Self {
        use SectionType::*;
        match value {
            Null => 0x0,
            Progbits => 0x1,
            Symtab => 0x2,
            Strtab => 0x3,
            Rela => 0x4,
            Hash => 0x5,
            Dynamic => 0x6,
            Note => 0x7,
            Nobits => 0x8,
            Rel => 0x9,
            Shlib => 0xA,
            Dynsym => 0xB,
            InitArray => 0xE,
            FiniArray => 0xF,
            PreinitArray => 0x10,
            Group => 0x11,
            SymtabShndx => 0x12,
            OsSpecific(x) => x,
            ProcessorSpecific(x) => x,
            User(x) => x,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, From, Into, BitXor, BitAnd, BitOr, LowerHex)]
pub struct SectionFlags32(pub u32);

impl SectionFlags32 {
    /// Writable.
    pub const WRITE: Self = Self(0x1);
    /// Occupies memory during execution.
    pub const ALLOC: Self = Self(0x2);
    /// Executable.
    pub const EXECINSTR: Self = Self(0x4);
    /// Might be merged.
    pub const MERGE: Self = Self(0x10);
    /// Contains nul-terminated strings.
    pub const STRINGS: Self = Self(0x20);
    /// `sh_info' contains SHT index.
    pub const INFOLINK: Self = Self(0x40);
    /// Preserve order after combining.
    pub const LINKORDER: Self = Self(0x80);
    /// Non-standard OS specific handling required.
    pub const OSNONCONFORMING: Self = Self(0x100);
    /// Section is member of a group.
    pub const GROUP: Self = Self(0x200);
    /// Section hold thread-local data.
    pub const TLS: Self = Self(0x400);
    /// Section with compressed data.
    pub const COMPRESSED: Self = Self(0x800);
    /// OS-specific.
    pub const MASKOS: Self = Self(0x0ff00000);
    /// Processor-specific.
    pub const MASKPROCESSOR: Self = Self(0xf0000000);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, From, Into, BitXor, BitAnd, BitOr, LowerHex)]
pub struct SectionFlags64(pub u64);

impl SectionFlags64 {
    /// Writable.
    pub const WRITE: Self = Self(0x1);
    /// Occupies memory during execution.
    pub const ALLOC: Self = Self(0x2);
    /// Executable.
    pub const EXECINSTR: Self = Self(0x4);
    /// Might be merged.
    pub const MERGE: Self = Self(0x10);
    /// Contains nul-terminated strings.
    pub const STRINGS: Self = Self(0x20);
    /// `sh_info' contains SHT index.
    pub const INFOLINK: Self = Self(0x40);
    /// Preserve order after combining.
    pub const LINKORDER: Self = Self(0x80);
    /// Non-standard OS specific handling required.
    pub const OSNONCONFORMING: Self = Self(0x100);
    /// Section is member of a group.
    pub const GROUP: Self = Self(0x200);
    /// Section hold thread-local data.
    pub const TLS: Self = Self(0x400);
    /// Section with compressed data.
    pub const COMPRESSED: Self = Self(0x800);
    /// OS-specific.
    pub const MASKOS: Self = Self(0x0ff00000);
    /// Processor-specific.
    pub const MASKPROCESSOR: Self = Self(0xf0000000);
}
