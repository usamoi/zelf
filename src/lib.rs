//! Zelf
//!
//! Zelf is a zero-allocation ELF parser in the "no_std" environment.
//!
//! Details:
//!
//! * Integers in the input structs are byte arrays so that all structs are of alignment one, and there will be no misaligned reading (it causes faults on some hardware).
//! * "context::Context" is a trait for parsing context, uniquely identified by class (32 / 64 bit), data encoding (little/big), version (current is 1) given by the identification in the elf header.
//!   It defines the layout of input structs (e.g. "ProgramHeader") and the output types (e.g. "context::Integer", "context::SectionFlags", "context::DynamicFlags").
//!   There are four combinations of them, which are four phantom types "Little32", "Little64", "Big32", "Big64".
//! * You can read "examples/readelf" for a starter with this crate.

#![no_std]

#[macro_use]
extern crate derive_more;

pub mod array;
pub mod context;
pub mod dynamic;
pub mod elf;
pub mod group;
pub mod hash;
pub mod ident;
pub mod interp;
pub mod note;
pub mod phdr;
pub mod program;
pub mod rel;
pub mod rela;
pub mod section;
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
