use crate::context::*;
use crate::utils::{read_s, Pod};

#[derive(Debug, Clone)]
pub enum ParseRelError {
    BrokenEntry,
}

/// Rel section.
#[derive(Debug, Clone, Copy)]
pub struct Rel<'a, T: Context> {
    entries: &'a [RelEntry<T>],
}

impl<'a, T: Context> Rel<'a, T> {
    pub fn parse(content: &'a [u8]) -> Result<Self, ParseRelError> {
        use ParseRelError::*;
        let entries = read_s(content).ok_or(BrokenEntry)?;
        Ok(Self { entries })
    }
    pub fn entries(&self) -> &'a [RelEntry<T>] {
        self.entries
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct RelEntry<T: Context> {
    pub offset: T::PropUsize,
    pub info: T::PropUsize,
}

impl<T: Context> RelEntry<T> {
    pub fn offset(&self) -> T::Integer {
        T::interpret(self.offset)
    }
    pub fn info(&self) -> T::Integer {
        T::interpret(self.info)
    }
}

unsafe impl<T: Context> Pod for RelEntry<T> {}
