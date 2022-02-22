use crate::interpret::*;
use crate::utils::{read_s, Pod};
use crate::{Integer, ParseError, Usize};

/// Rel section.
#[derive(Debug, Clone)]
pub struct Rel<'a, T: Interpreter> {
    entries: &'a [RelEntry<T>],
}

impl<'a, T: Interpreter> Rel<'a, T> {
    pub fn parse(content: &'a [u8]) -> Result<Self, ParseError> {
        use ParseError::*;
        let entries = read_s(content).ok_or(BrokenBody)?;
        Ok(Self { entries })
    }
    pub fn entries(&self) -> &'a [RelEntry<T>] {
        self.entries
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct RelEntry<T: Interpreter> {
    pub offset: Usize<T>,
    pub info: Usize<T>,
}

impl<T: Interpreter> RelEntry<T> {
    pub fn offset(&self) -> Integer<T> {
        T::interpret(self.offset)
    }
    pub fn info(&self) -> Integer<T> {
        T::interpret(self.info)
    }
}

unsafe impl<T: Interpreter> Pod for RelEntry<T> {}
