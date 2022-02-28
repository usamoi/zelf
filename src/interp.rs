use crate::utils::terminate;

#[derive(Debug, Clone)]
pub enum ParseInterpError {
    BadStringPath,
}

/// Interp program.
#[derive(Debug, Clone, Copy)]
pub struct Interp<'a> {
    path: &'a [u8],
}

impl<'a> Interp<'a> {
    pub fn parse(content: &'a [u8]) -> Result<Self, ParseInterpError> {
        use ParseInterpError::*;
        let path = terminate(content).ok_or(BadStringPath)?;
        Ok(Interp { path })
    }
    pub fn path(&self) -> &'a [u8] {
        self.path
    }
}
