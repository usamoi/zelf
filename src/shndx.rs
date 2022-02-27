use crate::context::*;
use crate::utils::*;
use core::marker::PhantomData;

#[derive(Debug, Clone)]
pub enum ParseShndxError {
    BadArray,
}

/// Symtab shndx section.
#[derive(Debug, Clone, Copy)]
pub struct Shndx<'a, T: Context> {
    entries: &'a [ShndxEntry<T>],
}

impl<'a, T: Context> Shndx<'a, T> {
    pub fn parse(content: &'a [u8]) -> Result<Self, ParseShndxError> {
        use ParseShndxError::*;
        let entries = read_s(content).ok_or(BadArray)?;
        Ok(Self { entries })
    }
    pub fn entries(&self) -> &'a [ShndxEntry<T>] {
        self.entries
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct ShndxEntry<T: Context> {
    pub value: PropU32,
    pub _maker: PhantomData<T>,
}

impl<T: Context> ShndxEntry<T> {
    pub fn value(&self) -> u32 {
        T::interpret(self.value)
    }
}

unsafe impl<T: Context> Pod for ShndxEntry<T> {}
