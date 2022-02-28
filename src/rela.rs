use crate::context::*;
use crate::utils::{read_s, Pod};

#[derive(Debug, Clone)]
pub enum ParseRelaError {
    BrokenEntry,
}

/// Rela section.
#[derive(Debug, Clone, Copy)]
pub struct Rela<'a, T: Context> {
    entries: &'a [RelaEntry<T>],
}

impl<'a, T: Context> Rela<'a, T> {
    pub fn parse(content: &'a [u8]) -> Result<Self, ParseRelaError> {
        use ParseRelaError::*;
        let entries = read_s(content).ok_or(BrokenEntry)?;
        Ok(Self { entries })
    }
    pub fn entries(&self) -> &'a [RelaEntry<T>] {
        self.entries
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct RelaEntry<T: Context> {
    pub offset: T::PropUsize,
    pub info: T::PropUsize,
    pub addend: T::PropUsize,
}

impl<T: Context> RelaEntry<T> {
    pub fn offset(&self) -> T::Integer {
        T::interpret(self.offset)
    }
    pub fn info(&self) -> T::Integer {
        T::interpret(self.info)
    }
    pub fn addend(&self) -> T::Integer {
        T::interpret(self.addend)
    }
}

unsafe impl<T: Context> Pod for RelaEntry<T> {}
