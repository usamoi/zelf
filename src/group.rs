use crate::context::PropU32;
use crate::context::*;
use crate::utils::*;
use core::marker::PhantomData;

#[derive(Debug, Clone)]
pub enum ParseGroupError {
    BadHeader,
    BadArray,
}

/// Group section.
#[derive(Debug, Clone, Copy)]
pub struct Group<'a, T: Context> {
    header: &'a GroupHeader<T>,
    entries: &'a [GroupEntry<T>],
}

impl<'a, T: Context> Group<'a, T> {
    pub fn parse(content: &'a [u8]) -> Result<Self, ParseGroupError> {
        use ParseGroupError::*;
        let mut offset = 0usize;
        let header = read(content, offset).ok_or(BadHeader)?;
        offset += core::mem::size_of::<GroupHeader<T>>();
        let entries = read_s::<GroupEntry<T>>(&content[offset..]).ok_or(BadArray)?;
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
pub struct GroupHeader<T: Context> {
    pub flags: PropU32,
    pub _maker: PhantomData<T>,
}

impl<T: Context> GroupHeader<T> {
    pub fn flags(&self) -> u32 {
        T::interpret(self.flags)
    }
}

unsafe impl<T: Context> Pod for GroupHeader<T> {}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct GroupEntry<T: Context> {
    pub index: PropU32,
    pub _maker: PhantomData<T>,
}

impl<T: Context> GroupEntry<T> {
    pub fn index(&self) -> u32 {
        T::interpret(self.index)
    }
}

unsafe impl<T: Context> Pod for GroupEntry<T> {}

#[derive(Debug, Clone, Copy, From, Into, BitAnd, BitOr, BitXor, LowerHex)]
pub struct GroupFlags(pub u32);

impl GroupFlags {
    pub const COMDAT: Self = Self(0x1);
    pub const MASKOS: Self = Self(0x0ff00000);
    pub const MASKPROCESSOR: Self = Self(0xf0000000);
}
