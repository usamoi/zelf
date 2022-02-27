use crate::context::*;
use crate::utils::*;

#[derive(Debug, Clone)]
pub enum ParseArrayError {
    BadArray,
}

/// Array section.
#[derive(Debug, Clone, Copy)]
pub struct Array<'a, T: Context> {
    entries: &'a [ArrayEntry<T>],
}

impl<'a, T: Context> Array<'a, T> {
    pub fn parse(content: &'a [u8]) -> Result<Self, ParseArrayError> {
        use ParseArrayError::*;
        let entries = read_s(content).ok_or(BadArray)?;
        Ok(Self { entries })
    }
    pub fn entries(&self) -> &'a [ArrayEntry<T>] {
        self.entries
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct ArrayEntry<T: Context> {
    pub value: T::PropUsize,
}

impl<T: Context> ArrayEntry<T> {
    pub fn value(&self) -> T::Integer {
        T::interpret(self.value)
    }
}

unsafe impl<T: Context> Pod for ArrayEntry<T> {}
