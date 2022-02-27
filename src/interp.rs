#[derive(Debug, Clone)]
pub enum ParseInterpError {
    BadString,
}

/// Interp program.
#[derive(Debug, Clone, Copy)]
pub struct Interp<'a> {
    path: &'a [u8],
}

impl<'a> Interp<'a> {
    pub fn parse(content: &'a [u8]) -> Result<Self, ParseInterpError> {
        use ParseInterpError::*;
        match content {
            [path @ .., 0] => {
                if path.iter().any(|c| *c == 0) {
                    return Err(BadString);
                }
                Ok(Interp { path })
            }
            _ => Err(BadString),
        }
    }
    pub fn path(&self) -> &'a [u8] {
        self.path
    }
}
