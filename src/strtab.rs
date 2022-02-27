#[derive(Debug, Clone)]
pub enum ParseStrtabError {
    BadString,
}

#[derive(Debug, Clone, Copy)]
pub struct Strtab<'a> {
    strings: &'a [u8],
}

impl<'a> Strtab<'a> {
    pub fn parse(content: &'a [u8]) -> Result<Self, ParseStrtabError> {
        use ParseStrtabError::*;
        match content {
            [.., 0] | [] => Ok(Self { strings: content }),
            [.., _] => Err(BadString),
        }
    }
    pub fn find(&self, name: usize) -> Option<&'a [u8]> {
        if name >= self.strings.len() {
            return None;
        }
        let mut pointer = name;
        while self.strings[pointer] != 0 {
            pointer += 1;
        }
        Some(&self.strings[name..pointer])
    }
    pub fn iter(&self) -> impl Iterator<Item = &'a [u8]> {
        enum Iter<'a> {
            Split(core::slice::Split<'a, u8, for<'r> fn(&'r u8) -> bool>),
            Empty(core::iter::Empty<&'a [u8]>),
        }
        impl<'a> Iterator for Iter<'a> {
            type Item = &'a [u8];

            fn next(&mut self) -> Option<Self::Item> {
                use Iter::*;
                match self {
                    Split(x) => x.next(),
                    Empty(x) => x.next(),
                }
            }
        }
        match self.strings {
            [s @ .., 0] => Iter::Split(s.split(|x: &u8| *x == 0)),
            [] => Iter::Empty(core::iter::empty()),
            _ => unreachable!(),
        }
    }
}
