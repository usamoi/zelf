use crate::interpret::*;
use crate::utils::*;
use crate::{ParseError, U32};
use core::marker::PhantomData;

/// Group section.
#[derive(Debug, Clone)]
pub struct Group<'a, T: Interpreter> {
    header: &'a GroupHeader<T>,
    entries: &'a [GroupEntry<T>],
}

impl<'a, T: Interpreter> Group<'a, T> {
    pub fn parse(content: &'a [u8]) -> Result<Self, ParseError> {
        use ParseError::*;
        let mut offset = 0usize;
        let header = read(content, offset).ok_or(BrokenHeader)?;
        offset += core::mem::size_of::<GroupHeader<T>>();
        let entries = read_s::<GroupEntry<T>>(&content[offset..]).ok_or(BrokenBody)?;
        Ok(Self { header, entries })
    }
    pub fn header(&self) -> &'a GroupHeader<T> {
        self.header
    }
    pub fn entries(&self) -> &'a [GroupEntry<T>] {
        self.entries
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct GroupHeader<T: Interpreter> {
    pub flags: U32,
    pub _maker: PhantomData<T>,
}

impl<T: Interpreter> GroupHeader<T> {
    pub fn flags(&self) -> u32 {
        T::interpret(self.flags)
    }
}

unsafe impl<T: Interpreter> Pod for GroupHeader<T> {}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct GroupEntry<T: Interpreter> {
    pub index: U32,
    pub _maker: PhantomData<T>,
}

impl<T: Interpreter> GroupEntry<T> {
    pub fn index(&self) -> u32 {
        T::interpret(self.index)
    }
}

unsafe impl<T: Interpreter> Pod for GroupEntry<T> {}

#[derive(Debug, Clone, Copy, From, Into, BitAnd, BitOr, BitXor, LowerHex)]
pub struct GroupFlags(pub u32);

impl GroupFlags {
    pub const COMDAT: Self = Self(0x1);
    pub const MASKOS: Self = Self(0x0ff00000);
    pub const MASKPROCESSOR: Self = Self(0xf0000000);
}
