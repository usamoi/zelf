#![no_std]

#[macro_use]
extern crate derive_more;

pub mod array;
pub mod dynamic;
pub mod elf;
pub mod group;
pub mod hash;
pub mod ident;
pub mod interp;
pub mod interpret;
pub mod note;
pub mod phdr;
pub mod program;
pub mod rel;
pub mod rela;
pub mod section;
pub mod strtab;
pub mod symtab;

mod utils;

/// ELF u16.
pub type U16 = [u8; 2];

/// ELF u32.
pub type U32 = [u8; 4];

/// ELF u64.
pub type U64 = [u8; 8];

/// ELF usize.
pub(crate) type Usize<T> = <T as utils::SealedInterpreter>::Usize;

/// ELF pointer-sized integer.
pub type Integer<T> = <T as interpret::Interpreter>::Integer;

/// ELF class.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Class {
    /// 32 bit.
    Class32 = 1,
    /// 64 bit.
    Class64 = 2,
}

impl TryFrom<u8> for Class {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        use Class::*;
        match value {
            1 => Ok(Class32),
            2 => Ok(Class64),
            _ => Err(()),
        }
    }
}

impl From<Class> for u8 {
    fn from(x: Class) -> Self {
        x as u8
    }
}

/// ELF data encoding.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Data {
    /// Little endian.
    Little = 1,
    /// Big endian.
    Big = 2,
}

impl TryFrom<u8> for Data {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        use Data::*;
        match value {
            1 => Ok(Little),
            2 => Ok(Big),
            _ => Err(()),
        }
    }
}

impl From<Data> for u8 {
    fn from(x: Data) -> Self {
        x as u8
    }
}

/// ELF version.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Version {
    /// Current.
    One = 1,
}

impl TryFrom<u8> for Version {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        use Version::*;
        match value {
            1 => Ok(One),
            _ => Err(()),
        }
    }
}

impl From<Version> for u8 {
    fn from(x: Version) -> Self {
        x as u8
    }
}

#[derive(Debug, Clone)]
pub enum ParseError {
    BrokenHeader,
    BrokenBody,
    BadProperty,
    BadString,
}
