use crate::context::*;
use crate::context::{PropU16, PropU32};
use crate::utils::{read_s, Pod};

#[derive(Debug, Clone)]
pub enum ParseSymtabError {
    BrokenEntry,
}

/// Symtab section.
#[derive(Debug, Clone, Copy)]
pub struct Symtab<'a, T: Context> {
    entries: &'a [SymtabEntry<T>],
}

impl<'a, T: Context> Symtab<'a, T> {
    pub fn parse(content: &'a [u8]) -> Result<Self, ParseSymtabError> {
        use ParseSymtabError::*;
        let entries = read_s(content).ok_or(BrokenEntry)?;
        Ok(Self { entries })
    }
    pub fn entries(&self) -> &'a [SymtabEntry<T>] {
        self.entries
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct SymtabEntry<T: Context> {
    pub name: PropU32,
    pub value32: T::PropUsizeIf32,
    pub size32: T::PropUsizeIf32,
    pub info: u8,
    pub other: u8,
    pub shndx: PropU16,
    pub value64: T::PropUsizeIf64,
    pub size64: T::PropUsizeIf64,
}

impl<T: Context> SymtabEntry<T> {
    pub fn name(&self) -> u32 {
        T::interpret(self.name)
    }
    pub fn value(&self) -> T::Integer {
        T::interpret((self.value32, self.value64))
    }
    pub fn size(&self) -> T::Integer {
        T::interpret((self.size32, self.size64))
    }
    pub fn info(&self) -> u8 {
        self.info
    }
    pub fn other(&self) -> u8 {
        self.other
    }
    pub fn shndx(&self) -> u16 {
        T::interpret(self.shndx)
    }
}

unsafe impl<T: Context> Pod for SymtabEntry<T> {}
