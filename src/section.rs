use crate::elf::ElfHeader;
use crate::interpret::*;
use crate::utils::{as_offset, read, read_n, Pod};
use crate::{Integer, ParseError, Usize, U32};
use core::marker::PhantomData;
use core::ops::RangeInclusive;

#[derive(Debug, Clone)]
pub struct Sections<'a, T: Interpreter> {
    data: &'a [u8],
    offset: usize,
    len: usize,
    _maker: PhantomData<T>,
}

impl<'a, T: Interpreter> Sections<'a, T> {
    pub(crate) fn parse(
        data: &'a [u8],
        eheader: &'a ElfHeader<T>,
    ) -> Result<Option<Sections<'a, T>>, ParseError> {
        use ParseError::*;
        let offset = as_offset::<T>(eheader.shoff()).ok_or(BadProperty)?;
        if offset == 0 {
            return Ok(None);
        }
        if eheader.shnum() as usize == 0 {
            // todo: follow the spec
            return Err(BadProperty);
        }
        if eheader.shentsize() as usize != core::mem::size_of::<SectionHeader<T>>() {
            return Err(BadProperty);
        }
        let len = eheader.shnum() as usize;
        read_n::<SectionHeader<T>>(data, offset, len).ok_or(BadProperty)?;
        // section zero
        let z: Section<T> = Section::parse(data, offset).map_err(|_| BadProperty)?;
        if z.header().name() != 0 {
            return Err(BadProperty);
        }
        if z.header().typa() != SectionType::Null {
            return Err(BadProperty);
        }
        if z.header().flags().into() != 0u32.into() {
            return Err(BadProperty);
        }
        if as_offset::<T>(z.header().addr()).ok_or(BadProperty)? != 0 {
            return Err(BadProperty);
        }
        if as_offset::<T>(z.header().offset()).ok_or(BadProperty)? != 0 {
            return Err(BadProperty);
        }
        let z_size = as_offset::<T>(z.header().size()).ok_or(BadProperty)?;
        if z_size != 0 && z_size != eheader.shnum() as usize {
            return Err(BadProperty);
        }
        if z.header().link() != 0 && z.header().link() != eheader.shstrndx() as u32 {
            return Err(BadProperty);
        }
        if z.header().info() != 0 {
            return Err(BadProperty);
        }
        if as_offset::<T>(z.header().addralign()).ok_or(BadProperty)? != 0 {
            return Err(BadProperty);
        }
        if as_offset::<T>(z.header().entsize()).ok_or(BadProperty)? != 0 {
            return Err(BadProperty);
        }
        Ok(Some(Self {
            data,
            offset,
            len,
            _maker: PhantomData,
        }))
    }
    pub fn len(&self) -> usize {
        self.len
    }
    pub fn get(&self, index: usize) -> Option<Result<Section<'a, T>, ParseError>> {
        if index >= self.len {
            return None;
        }
        let offset = self.offset + index * core::mem::size_of::<SectionHeader<T>>();
        Some(Section::parse(self.data, offset))
    }
    pub fn iter(&self) -> impl Iterator<Item = Result<Section<'a, T>, ParseError>> + 'a {
        let data = self.data;
        let offset = self.offset;
        let len = self.len;
        let mut index = 0usize;
        core::iter::from_fn(move || {
            if index >= len {
                return None;
            }
            let offset = offset + index * core::mem::size_of::<SectionHeader<T>>();
            let ans = Section::parse(data, offset);
            index += 1;
            Some(ans)
        })
    }
}

pub struct Section<'a, T: Interpreter> {
    sheader: &'a SectionHeader<T>,
    content: &'a [u8],
}

impl<'a, T: Interpreter> Section<'a, T> {
    pub(crate) fn parse(data: &'a [u8], offset: usize) -> Result<Self, ParseError> {
        use ParseError::*;
        use SectionType::*;
        let sheader: &'a SectionHeader<T> = read(data, offset).ok_or(BrokenHeader)?;
        let typa = SectionType::try_from(T::interpret(sheader.typa)).map_err(|_| BadProperty)?;
        match typa {
            Null => Ok(Section {
                sheader,
                content: &[],
            }),
            Nobits => {
                let content_offset = as_offset::<T>(sheader.offset()).ok_or(BrokenBody)?;
                let content = read_n::<u8>(data, content_offset, 0).ok_or(BrokenBody)?;
                Ok(Section { sheader, content })
            }
            _ => {
                let content_offset = as_offset::<T>(sheader.offset()).ok_or(BrokenBody)?;
                let content_size = as_offset::<T>(sheader.size()).ok_or(BrokenBody)?;
                let content = read_n::<u8>(data, content_offset, content_size).ok_or(BrokenBody)?;
                Ok(Section { sheader, content })
            }
        }
    }
    pub fn header(&self) -> &'a SectionHeader<T> {
        self.sheader
    }
    pub fn content(&self) -> &'a [u8] {
        self.content
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct SectionHeader<T: Interpreter> {
    pub name: U32,
    pub typa: U32,
    pub flags: Usize<T>,
    pub addr: Usize<T>,
    pub offset: Usize<T>,
    pub size: Usize<T>,
    pub link: U32,
    pub info: U32,
    pub addralign: Usize<T>,
    pub entsize: Usize<T>,
}

impl<T: Interpreter> SectionHeader<T> {
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
        From::<Integer<T>>::from(T::interpret(self.flags))
    }
    pub fn addr(&self) -> Integer<T> {
        T::interpret(self.addr)
    }
    pub fn offset(&self) -> Integer<T> {
        T::interpret(self.offset)
    }
    pub fn size(&self) -> Integer<T> {
        T::interpret(self.size)
    }
    pub fn link(&self) -> u32 {
        T::interpret(self.link)
    }
    pub fn info(&self) -> u32 {
        T::interpret(self.info)
    }
    pub fn addralign(&self) -> Integer<T> {
        T::interpret(self.addralign)
    }
    pub fn entsize(&self) -> Integer<T> {
        T::interpret(self.entsize)
    }
}

unsafe impl<T: Interpreter> Pod for SectionHeader<T> {}

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
