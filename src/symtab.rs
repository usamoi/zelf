use crate::interpret::*;
use crate::utils::{read_s, Pod};
use crate::{Integer, ParseError, U16, U32};

/// Symtab section.
#[derive(Debug, Clone)]
pub struct Symtab<'a, T: Interpreter> {
    entries: &'a [SymtabEntry<T>],
}

impl<'a, T: Interpreter> Symtab<'a, T> {
    pub fn parse(content: &'a [u8]) -> Result<Self, ParseError> {
        use ParseError::*;
        let entries = read_s(content).ok_or(BrokenBody)?;
        Ok(Self { entries })
    }
    pub fn entries(&self) -> &'a [SymtabEntry<T>] {
        self.entries
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct SymtabEntry<T: Interpreter> {
    pub name: U32,
    pub value32: T::PropUsizeIf32,
    pub size32: T::PropUsizeIf32,
    pub info: u8,
    pub other: u8,
    pub shndx: U16,
    pub value64: T::PropUsizeIf64,
    pub size64: T::PropUsizeIf64,
}

impl<T: Interpreter> SymtabEntry<T> {
    pub fn name(&self) -> u32 {
        T::interpret(self.name)
    }
    pub fn value(&self) -> Integer<T> {
        T::interpret((self.value32, self.value64))
    }
    pub fn size(&self) -> Integer<T> {
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

unsafe impl<T: Interpreter> Pod for SymtabEntry<T> {}
