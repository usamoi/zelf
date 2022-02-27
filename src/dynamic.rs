use crate::context::*;
use crate::utils::{as_offset, read_s, Pod};

#[derive(Debug, Clone)]
pub enum ParseDynamicError {
    BadArray,
    BadPropertyTag,
}

/// Dynamic section/program.
#[derive(Debug, Clone, Copy)]
pub struct Dynamic<'a, T: Context> {
    entries: &'a [DynamicEntry<T>],
}

impl<'a, T: Context> Dynamic<'a, T> {
    pub fn parse(content: &'a [u8]) -> Result<Self, ParseDynamicError> {
        use ParseDynamicError::*;
        let entries: &[DynamicEntry<T>] = read_s(content).ok_or(BadArray)?;
        for entry in entries {
            let _tag = entry.checked_tag().ok_or(BadPropertyTag)?;
        }
        Ok(Self { entries })
    }
    pub fn entries(&self) -> &'a [DynamicEntry<T>] {
        self.entries
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct DynamicEntry<T: Context> {
    pub tag: T::PropUsize,
    pub un: T::PropUsize,
}

impl<T: Context> DynamicEntry<T> {
    pub fn checked_tag(&self) -> Option<DynamicTag> {
        let value = as_offset::<T>(T::interpret(self.tag))?;
        TryInto::<u32>::try_into(value).ok()?.try_into().ok()
    }
    /// # Panics
    ///
    /// Panics if it's an invaild value.
    pub fn tag(&self) -> DynamicTag {
        self.checked_tag().unwrap()
    }
    pub fn un(&self) -> T::Integer {
        T::interpret(self.un)
    }
}

unsafe impl<T: Context> Pod for DynamicEntry<T> {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DynamicTag {
    Null,
    Needed,
    PltRelSize,
    PltGot,
    Hash,
    Strtab,
    Symtab,
    Rela,
    RelaSize,
    RelaEnt,
    StrSize,
    SymEnt,
    Init,
    Fini,
    SoName,
    RPath,
    Symbolic,
    Rel,
    RelSize,
    RelEnt,
    PltRel,
    Debug,
    TextRel,
    JmpRel,
    BindNow,
    InitArray,
    FiniArray,
    InitArraySize,
    FiniArraySize,
    RunPath,
    Flags,
    PreInitArray,
    PreInitArraySize,
    SymtabShndx,
    OsSpecific(u32),
    ProcessorSpecific(u32),
}

impl TryFrom<u32> for DynamicTag {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        use DynamicTag::*;
        match value {
            0 => Ok(Null),
            1 => Ok(Needed),
            2 => Ok(PltRelSize),
            3 => Ok(PltGot),
            4 => Ok(Hash),
            5 => Ok(Strtab),
            6 => Ok(Symtab),
            7 => Ok(Rela),
            8 => Ok(RelaSize),
            9 => Ok(RelaEnt),
            10 => Ok(StrSize),
            11 => Ok(SymEnt),
            12 => Ok(Init),
            13 => Ok(Fini),
            14 => Ok(SoName),
            15 => Ok(RPath),
            16 => Ok(Symbolic),
            17 => Ok(Rel),
            18 => Ok(RelSize),
            19 => Ok(RelEnt),
            20 => Ok(PltRel),
            21 => Ok(Debug),
            22 => Ok(TextRel),
            23 => Ok(JmpRel),
            24 => Ok(BindNow),
            25 => Ok(InitArray),
            26 => Ok(FiniArray),
            27 => Ok(InitArraySize),
            28 => Ok(FiniArraySize),
            29 => Ok(RunPath),
            30 => Ok(Flags),
            32 => Ok(PreInitArray),
            33 => Ok(PreInitArraySize),
            34 => Ok(SymtabShndx),
            x @ 0x6000000D..=0x6FFFF000 => Ok(OsSpecific(x)),
            x @ 0x70000000..=0x7FFFFFFF => Ok(ProcessorSpecific(x)),
            _ => Err(()),
        }
    }
}

impl From<DynamicTag> for u32 {
    fn from(x: DynamicTag) -> Self {
        use DynamicTag::*;
        match x {
            Null => 0,
            Needed => 1,
            PltRelSize => 2,
            PltGot => 3,
            Hash => 4,
            Strtab => 5,
            Symtab => 6,
            Rela => 7,
            RelaSize => 8,
            RelaEnt => 9,
            StrSize => 10,
            SymEnt => 11,
            Init => 12,
            Fini => 13,
            SoName => 14,
            RPath => 15,
            Symbolic => 16,
            Rel => 17,
            RelSize => 18,
            RelEnt => 19,
            PltRel => 20,
            Debug => 21,
            TextRel => 22,
            JmpRel => 23,
            BindNow => 24,
            InitArray => 25,
            FiniArray => 26,
            InitArraySize => 27,
            FiniArraySize => 28,
            RunPath => 29,
            Flags => 30,
            PreInitArray => 32,
            PreInitArraySize => 33,
            SymtabShndx => 34,
            OsSpecific(x) => x,
            ProcessorSpecific(x) => x,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, From, Into, BitAnd, BitOr, BitXor, LowerHex)]
pub struct DynamicFlags32(pub u32);

impl DynamicFlags32 {
    pub const ORIGIN: Self = Self(0x1);
    pub const SYMBOLIC: Self = Self(0x2);
    pub const TEXTREL: Self = Self(0x4);
    pub const BIND_NOW: Self = Self(0x8);
    pub const STATIC_TLS: Self = Self(0x10);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, From, Into, BitAnd, BitOr, BitXor, LowerHex)]
pub struct DynamicFlags64(pub u64);

impl DynamicFlags64 {
    pub const ORIGIN: Self = Self(0x1);
    pub const SYMBOLIC: Self = Self(0x2);
    pub const TEXTREL: Self = Self(0x4);
    pub const BIND_NOW: Self = Self(0x8);
    pub const STATIC_TLS: Self = Self(0x10);
}
