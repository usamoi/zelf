use crate::{utils::check_string, ParseError};

/// Interp program.
#[derive(Debug, Clone)]
pub struct Interp<'a> {
    path: &'a [u8],
}

impl<'a> Interp<'a> {
    pub fn parse(content: &'a [u8]) -> Result<Self, ParseError> {
        use ParseError::*;
        match content {
            [path @ .., 0] => {
                check_string(path)?;
                Ok(Interp { path })
            }
            _ => Err(BadString),
        }
    }
    pub fn path(&self) -> &'a [u8] {
        self.path
    }
}
