//! Zelf is a zero-allocation ELF parser designed for the "no_std" environment.
//!
//! It defines ELF C structs and provides parsing functions and parsed Rust types.
//! Fields in ELF C structs are byte arrays so that all structs are of alignment one to escape misaligned reading (it causes faults on some hardware).
//!
//! "context::Context" is a trait for parsing context, uniquely identified by class (32/64 bit), data encoding (little/big), version (current is 1) given by the identification in the elf header.
//! It determines the layout of ELF structs (e.g. "ProgramHeader", "ArrayEntry") and the parsed Rust types (e.g. "Context::Integer", "Context::SectionFlags", "Context::DynamicFlags").
//! There are four combinations of them, which are four phantom types "Little32", "Little64", "Big32", "Big64".
//!
//! You need to call the corresponding parsing functions for sections and programs. There is a table for reference.
//!
//! | Section/Program Type                                  | parsing function |
//! |-------------------------------------------------------|------------------|
//! | Null, Probits, Nobits, Shlib, Load, Phdr, Tls         | N/A              |
//! | Symtab, Dynsym                                        | Symtab::parse    |
//! | Strtab, Rela, Hash, Dynamic, Note, Rel, Group, Interp | {type}::parse    |
//! | InitArray, FiniArray, PreinitArray                    | Array::parse     |
//! | SymtabShndx                                           | Shndx::parse     |
//!
//! You need to call "Compression::parse" for compressed sections.
//!
//! You can read "examples/readelf" for a starter with this crate.

#![no_std]

#[macro_use]
extern crate derive_more;

pub mod array;
pub mod compression;
pub mod context;
pub mod dynamic;
pub mod elf;
pub mod group;
pub mod hash;
pub mod ident;
pub mod interp;
pub mod note;
pub mod program;
pub mod rel;
pub mod rela;
pub mod section;
pub mod shndx;
pub mod strtab;
pub mod symtab;

mod utils;

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
    fn from(value: Class) -> Self {
        value as u8
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
    fn from(value: Data) -> Self {
        value as u8
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
    fn from(value: Version) -> Self {
        value as u8
    }
}
