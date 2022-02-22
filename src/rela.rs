use crate::interpret::*;
use crate::utils::{read_s, Pod};
use crate::{Integer, ParseError, Usize};

/// Rela section.
#[derive(Debug, Clone)]
pub struct Rela<'a, T: Interpreter> {
    entries: &'a [RelaEntry<T>],
}

impl<'a, T: Interpreter> Rela<'a, T> {
    pub fn parse(content: &'a [u8]) -> Result<Self, ParseError> {
        use ParseError::*;
        let entries = read_s(content).ok_or(BrokenBody)?;
        Ok(Self { entries })
    }
    pub fn entries(&self) -> &'a [RelaEntry<T>] {
        self.entries
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct RelaEntry<T: Interpreter> {
    pub offset: Usize<T>,
    pub info: Usize<T>,
    pub addend: Usize<T>,
}

impl<T: Interpreter> RelaEntry<T> {
    pub fn offset(&self) -> Integer<T> {
        T::interpret(self.offset)
    }
    pub fn info(&self) -> Integer<T> {
        T::interpret(self.info)
    }
    pub fn addend(&self) -> Integer<T> {
        T::interpret(self.addend)
    }
}

unsafe impl<T: Interpreter> Pod for RelaEntry<T> {}
