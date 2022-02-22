use crate::utils::{align, check_string, read, read_n, Pod};
use crate::{interpret::*, Integer};
use crate::{ParseError, U32};
use core::marker::PhantomData;

/// Note section/program.
#[derive(Debug, Clone)]
pub struct Note<'a> {
    typa: u32,
    name: &'a [u8],
    descriptor: &'a [u8],
}

impl<'a> Note<'a> {
    #[allow(unused_assignments)]
    pub fn parse<T: Interpreter>(content: &'a [u8]) -> Result<Self, ParseError> {
        use ParseError::*;
        let mut offset = 0usize;
        let header: &NoteHeader<T> = read(content, offset).ok_or(BrokenHeader)?;
        offset += core::mem::size_of::<NoteHeader<T>>();
        let name: &[u8] = read_n(content, offset, header.name_size() as usize).ok_or(BrokenBody)?;
        offset += header.name_size() as usize;
        offset = align(offset, core::mem::align_of::<Integer<T>>());
        let descriptor =
            read_n::<u8>(content, offset, header.descriptor_size() as usize).ok_or(BrokenBody)?;
        offset += header.descriptor_size() as usize;
        // seems no need to check if it's ending
        check_string(name)?;
        Ok(Self {
            typa: header.typa(),
            name,
            descriptor,
        })
    }
    pub fn typa(&self) -> u32 {
        self.typa
    }
    pub fn name(&self) -> &'a [u8] {
        self.name
    }
    pub fn descriptor(&self) -> &'a [u8] {
        self.descriptor
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct NoteHeader<T: Interpreter> {
    pub name_size: U32,
    pub descriptor_size: U32,
    pub typa: U32,
    pub _maker: PhantomData<T>,
}

impl<T: Interpreter> NoteHeader<T> {
    pub fn name_size(&self) -> u32 {
        T::interpret(self.name_size)
    }
    pub fn descriptor_size(&self) -> u32 {
        T::interpret(self.descriptor_size)
    }
    pub fn typa(&self) -> u32 {
        T::interpret(self.typa)
    }
}

unsafe impl<T: Interpreter> Pod for NoteHeader<T> {}
