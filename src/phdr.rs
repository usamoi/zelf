use crate::interpret::*;
use crate::program::ProgramHeader;
use crate::utils::*;
use crate::ParseError;

/// Phdr program.
#[derive(Debug, Clone)]
pub struct Phdr<'a, T: Interpreter> {
    entries: &'a [ProgramHeader<T>],
}

impl<'a, T: Interpreter> Phdr<'a, T> {
    pub fn parse(content: &'a [u8]) -> Result<Self, ParseError> {
        use ParseError::*;
        let entries = read_s(content).ok_or(BrokenBody)?;
        Ok(Self { entries })
    }
    pub fn entries(&self) -> &'a [ProgramHeader<T>] {
        self.entries
    }
}
