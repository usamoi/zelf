use crate::context::PropU32;
use crate::context::*;
use crate::elf::Elf;
use crate::strtab::Strtab;
use crate::utils::{as_offset, read, read_n, Pod};
use core::marker::PhantomData;
use core::ops::RangeInclusive;

#[derive(Debug, Clone)]
pub enum ParseSectionsError {
    BadPropertyShentsize,
    BadSectionHeaders,
    BadSectionZero,
    BadArray,
}

#[derive(Debug, Clone)]
pub enum ParseSectionError {
    BadHeader,
    BadPropertyType,
    BadContent,
}

#[derive(Debug, Clone)]
pub enum ParseShstrtabError {
    BadPropertyShstridx,
    BadSection,
    BadPropertyType,
    BadStrtab,
}

#[derive(Debug, Clone, Copy)]
pub struct Sections<'a, T: Context> {
    data: &'a [u8],
    shstrndx: u16,
    offset: usize,
    num: u16,
    _maker: PhantomData<T>,
}

impl<'a, T: Context> Sections<'a, T> {
    pub fn parse(elf: Elf<'a, T>) -> Result<Option<Sections<'a, T>>, ParseSectionsError> {
        use ParseSectionsError::*;
        if elf.header().shentsize() as usize != core::mem::size_of::<SectionHeader<T>>() {
            return Err(BadPropertyShentsize);
        }
        let offset = as_offset::<T>(elf.header().shoff()).ok_or(BadSectionHeaders)?;
        if offset == 0 {
            return Ok(None);
        }
        if elf.header().shnum() as usize == 0 {
            // todo: follow the spec
            return Err(BadSectionHeaders);
        }
        let num = elf.header().shnum();
        read_n::<SectionHeader<T>>(elf.data(), offset, num as usize).ok_or(BadSectionHeaders)?;
        let sections = Self {
            data: elf.data(),
            offset,
            shstrndx: elf.header().shstrndx(),
            num,
            _maker: PhantomData,
        };
        // section zero
        let zero: Section<T> = Section::parse(sections, 0)
            .unwrap()
            .map_err(|_| BadSectionZero)?;
        if zero.header().name() != 0 {
            return Err(BadSectionZero);
        }
        if zero.header().typa() != SectionType::Null {
            return Err(BadSectionZero);
        }
        if zero.header().flags().into() != 0u32.into() {
            return Err(BadSectionZero);
        }
        if as_offset::<T>(zero.header().addr()).ok_or(BadSectionZero)? != 0 {
            return Err(BadSectionZero);
        }
        if as_offset::<T>(zero.header().offset()).ok_or(BadSectionZero)? != 0 {
            return Err(BadSectionZero);
        }
        let zero_size = as_offset::<T>(zero.header().size()).ok_or(BadSectionZero)?;
        if zero_size != 0 && zero_size != elf.header().shnum() as usize {
            return Err(BadSectionZero);
        }
        if zero.header().link() != 0 && zero.header().link() != elf.header().shstrndx() as u32 {
            return Err(BadSectionZero);
        }
        if zero.header().info() != 0 {
            return Err(BadSectionZero);
        }
        if as_offset::<T>(zero.header().addralign()).ok_or(BadSectionZero)? != 0 {
            return Err(BadSectionZero);
        }
        if as_offset::<T>(zero.header().entsize()).ok_or(BadSectionZero)? != 0 {
            return Err(BadSectionZero);
        }
        Ok(Some(sections))
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
            let sheader: &'a SectionHeader<T> = read(data, offset).ok_or(BadHeader)?;
            let typa =
                SectionType::try_from(T::interpret(sheader.typa)).map_err(|_| BadPropertyType)?;
            match typa {
                Null => Ok(Section {
                    sheader,
                    content: &[],
                }),
                Nobits => {
                    let content_offset = as_offset::<T>(sheader.offset()).ok_or(BadContent)?;
                    let content = read_n::<u8>(data, content_offset, 0).ok_or(BadContent)?;
                    Ok(Section { sheader, content })
                }
                _ => {
                    let content_offset = as_offset::<T>(sheader.offset()).ok_or(BadContent)?;
                    let content_size = as_offset::<T>(sheader.size()).ok_or(BadContent)?;
                    let content =
                        read_n::<u8>(data, content_offset, content_size).ok_or(BadContent)?;
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
            .ok_or(BadPropertyShstridx)?
            .map_err(|_| BadSection)?;
        if section.header().typa() != SectionType::Strtab {
            return Err(BadPropertyType);
        }
        let shstrtab = Strtab::parse(section.content()).map_err(|_| BadStrtab)?;
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
pub const SECTION_INDEX_PROCESSORSPECIFIC: RangeInclusive<u16> = (0xff00)..=(0xff1f);
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
