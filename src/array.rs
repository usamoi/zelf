use crate::interpret::*;
use crate::utils::*;
use crate::ParseError;
use crate::{Integer, Usize};

/// Array section.
#[derive(Debug, Clone)]
pub struct Array<'a, T: Interpreter> {
    entries: &'a [ArrayEntry<T>],
}

impl<'a, T: Interpreter> Array<'a, T> {
    pub fn parse(content: &'a [u8]) -> Result<Self, ParseError> {
        use ParseError::*;
        let entries = read_s(content).ok_or(BrokenBody)?;
        Ok(Self { entries })
    }
    pub fn entries(&self) -> &'a [ArrayEntry<T>] {
        self.entries
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct ArrayEntry<T: Interpreter> {
    pub value: Usize<T>,
}

impl<T: Interpreter> ArrayEntry<T> {
    pub fn value(&self) -> Integer<T> {
        T::interpret(self.value)
    }
}

unsafe impl<T: Interpreter> Pod for ArrayEntry<T> {}
