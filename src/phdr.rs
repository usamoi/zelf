use crate::context::*;
use crate::program::ProgramHeader;
use crate::utils::*;

#[derive(Debug, Clone)]
pub enum ParsePhdrError {
    BadArray,
}

/// Phdr program.
#[derive(Debug, Clone, Copy)]
pub struct Phdr<'a, T: Context> {
    entries: &'a [ProgramHeader<T>],
}

impl<'a, T: Context> Phdr<'a, T> {
    pub fn parse(content: &'a [u8]) -> Result<Self, ParsePhdrError> {
        use ParsePhdrError::*;
        let entries = read_s(content).ok_or(BadArray)?;
        Ok(Self { entries })
    }
    pub fn entries(&self) -> &'a [ProgramHeader<T>] {
        self.entries
    }
}
