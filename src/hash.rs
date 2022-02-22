use crate::interpret::*;
use crate::utils::{read, read_n, Pod};
use crate::{ParseError, U32};
use core::marker::PhantomData;

/// Hash section.
#[derive(Debug, Clone)]
pub struct Hash<'a, T: Interpreter> {
    buckets: &'a [HashBucketEntry<T>],
    chains: &'a [HashChainEntry<T>],
}

impl<'a, T: Interpreter> Hash<'a, T> {
    pub fn parse(content: &'a [u8]) -> Result<Self, ParseError> {
        use ParseError::*;
        let mut offset = 0usize;
        let header: &HashHeader<T> = read(content, offset).ok_or(BrokenHeader)?;
        let buckets: &[HashBucketEntry<T>] =
            read_n(content, offset, header.nbuckets() as usize).ok_or(BrokenBody)?;
        offset += core::mem::size_of::<HashBucketEntry<T>>() * header.nbuckets() as usize;
        let chains: &[HashChainEntry<T>] =
            read_n(content, offset, header.nchains() as usize).ok_or(BrokenBody)?;
        offset += core::mem::size_of::<HashChainEntry<T>>() * header.nbuckets() as usize;
        if offset != content.len() {
            return Err(BrokenBody);
        }
        Ok(Self { buckets, chains })
    }
    pub fn buckets(&self) -> &'a [HashBucketEntry<T>] {
        self.buckets
    }
    pub fn chains(&self) -> &'a [HashChainEntry<T>] {
        self.chains
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct HashHeader<T: Interpreter> {
    pub nbuckets: U32,
    pub nchains: U32,
    pub _maker: PhantomData<T>,
}

impl<T: Interpreter> HashHeader<T> {
    pub fn nbuckets(&self) -> u32 {
        T::interpret(self.nbuckets)
    }
    pub fn nchains(&self) -> u32 {
        T::interpret(self.nchains)
    }
}

unsafe impl<T: Interpreter> Pod for HashHeader<T> {}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct HashBucketEntry<T: Interpreter> {
    pub value: U32,
    pub _maker: PhantomData<T>,
}

impl<T: Interpreter> HashBucketEntry<T> {
    pub fn value(&self) -> u32 {
        T::interpret(self.value)
    }
}

unsafe impl<T: Interpreter> Pod for HashBucketEntry<T> {}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct HashChainEntry<T: Interpreter> {
    pub value: U32,
    pub _maker: PhantomData<T>,
}

impl<T: Interpreter> HashChainEntry<T> {
    pub fn value(&self) -> u32 {
        T::interpret(self.value)
    }
}

unsafe impl<T: Interpreter> Pod for HashChainEntry<T> {}

/// ELF hash function.
#[allow(arithmetic_overflow)]
pub fn hash(name: &[u8]) -> u32 {
    let mut r = 0u32;
    for x in name.iter().copied() {
        r = (r << 4) + x as u32;
        let g = r & 0xf0000000;
        if g != 0 {
            r ^= g >> 24;
        }
        r &= !g;
    }
    r
}
