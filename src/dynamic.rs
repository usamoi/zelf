use crate::context::*;
use crate::utils::{read_s, Pod};

#[derive(Debug, Clone)]
pub enum ParseDynamicError {
    BadArray,
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
    pub fn tag(&self) -> T::DynamicTag {
        From::<T::Integer>::from(T::interpret(self.tag))
    }
    pub fn un(&self) -> T::Integer {
        T::interpret(self.un)
    }
}

unsafe impl<T: Context> Pod for DynamicEntry<T> {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DynamicTag32 {
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
    NonStandard(u32),
}

impl From<u32> for DynamicTag32 {
    fn from(value: u32) -> Self {
        use DynamicTag32::*;
        match value {
            0 => Null,
            1 => Needed,
            2 => PltRelSize,
            3 => PltGot,
            4 => Hash,
            5 => Strtab,
            6 => Symtab,
            7 => Rela,
            8 => RelaSize,
            9 => RelaEnt,
            10 => StrSize,
            11 => SymEnt,
            12 => Init,
            13 => Fini,
            14 => SoName,
            15 => RPath,
            16 => Symbolic,
            17 => Rel,
            18 => RelSize,
            19 => RelEnt,
            20 => PltRel,
            21 => Debug,
            22 => TextRel,
            23 => JmpRel,
            24 => BindNow,
            25 => InitArray,
            26 => FiniArray,
            27 => InitArraySize,
            28 => FiniArraySize,
            29 => RunPath,
            30 => Flags,
            32 => PreInitArray,
            33 => PreInitArraySize,
            34 => SymtabShndx,
            x @ 0x6000000D..=0x6FFFF000 => OsSpecific(x),
            x @ 0x70000000..=0x7FFFFFFF => ProcessorSpecific(x),
            x => NonStandard(x),
        }
    }
}

impl From<DynamicTag32> for u32 {
    fn from(value: DynamicTag32) -> Self {
        use DynamicTag32::*;
        match value {
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
            NonStandard(x) => x,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DynamicTag64 {
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
    OsSpecific(u64),
    ProcessorSpecific(u64),
    NonStandard(u64),
}

impl From<u64> for DynamicTag64 {
    fn from(value: u64) -> Self {
        use DynamicTag64::*;
        match value {
            0 => Null,
            1 => Needed,
            2 => PltRelSize,
            3 => PltGot,
            4 => Hash,
            5 => Strtab,
            6 => Symtab,
            7 => Rela,
            8 => RelaSize,
            9 => RelaEnt,
            10 => StrSize,
            11 => SymEnt,
            12 => Init,
            13 => Fini,
            14 => SoName,
            15 => RPath,
            16 => Symbolic,
            17 => Rel,
            18 => RelSize,
            19 => RelEnt,
            20 => PltRel,
            21 => Debug,
            22 => TextRel,
            23 => JmpRel,
            24 => BindNow,
            25 => InitArray,
            26 => FiniArray,
            27 => InitArraySize,
            28 => FiniArraySize,
            29 => RunPath,
            30 => Flags,
            32 => PreInitArray,
            33 => PreInitArraySize,
            34 => SymtabShndx,
            x @ 0x6000000D..=0x6FFFF000 => OsSpecific(x),
            x @ 0x70000000..=0x7FFFFFFF => ProcessorSpecific(x),
            x => NonStandard(x),
        }
    }
}

impl From<DynamicTag64> for u64 {
    fn from(value: DynamicTag64) -> Self {
        use DynamicTag64::*;
        match value {
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
            NonStandard(x) => x,
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
