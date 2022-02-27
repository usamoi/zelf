use crate::context::PropU32;
use crate::context::*;
use crate::utils::{align, read, read_n, Pod};
use core::marker::PhantomData;

#[derive(Debug, Clone)]
pub enum ParseNoteError {
    BadHeader,
    BadName,
    BadDescriptor,
    BadString,
}

/// Note section/program.
#[derive(Debug, Clone, Copy)]
pub struct Note<'a> {
    typa: u32,
    name: &'a [u8],
    descriptor: &'a [u8],
}

impl<'a> Note<'a> {
    #[allow(unused_assignments)]
    pub fn parse<T: Context>(content: &'a [u8]) -> Result<Self, ParseNoteError> {
        use ParseNoteError::*;
        let mut offset = 0usize;
        let header: &NoteHeader<T> = read(content, offset).ok_or(BadHeader)?;
        offset += core::mem::size_of::<NoteHeader<T>>();
        let name: &[u8] = read_n(content, offset, header.name_size() as usize).ok_or(BadName)?;
        offset += header.name_size() as usize;
        offset = align(offset, core::mem::align_of::<T::Integer>());
        let descriptor = read_n::<u8>(content, offset, header.descriptor_size() as usize)
            .ok_or(BadDescriptor)?;
        offset += header.descriptor_size() as usize;
        // seems no need to check if it's ending
        if name.iter().any(|c| *c == 0) {
            return Err(BadString);
        }
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
pub struct NoteHeader<T: Context> {
    pub name_size: PropU32,
    pub descriptor_size: PropU32,
    pub typa: PropU32,
    pub _maker: PhantomData<T>,
}

impl<T: Context> NoteHeader<T> {
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

unsafe impl<T: Context> Pod for NoteHeader<T> {}
