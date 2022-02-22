use crate::elf::ElfHeader;
use crate::interpret::*;
use crate::utils::{as_offset, read, read_n, Pod};
use crate::{Integer, ParseError, Usize, U32};
use core::marker::PhantomData;

#[derive(Debug, Clone)]
pub struct Programs<'a, T: Interpreter> {
    data: &'a [u8],
    offset: usize,
    len: usize,
    _maker: PhantomData<T>,
}

impl<'a, T: Interpreter> Programs<'a, T> {
    pub(crate) fn parse(
        data: &'a [u8],
        eheader: &'a ElfHeader<T>,
    ) -> Result<Option<Programs<'a, T>>, ParseError> {
        use ParseError::*;
        let offset = as_offset::<T>(eheader.phoff()).ok_or(BadProperty)?;
        if offset == 0 {
            return Ok(None);
        }
        if eheader.phentsize() as usize != core::mem::size_of::<ProgramHeader<T>>() {
            return Err(BadProperty);
        }
        let len = eheader.phnum() as usize;
        read_n::<ProgramHeader<T>>(data, offset, len).ok_or(BadProperty)?;
        Ok(Some(Self {
            data,
            offset,
            len,
            _maker: PhantomData,
        }))
    }
    pub fn len(&self) -> usize {
        self.len
    }
    pub fn get(&self, index: usize) -> Option<Result<Program<'a, T>, ParseError>> {
        if index >= self.len {
            return None;
        }
        let offset = self.offset + index * core::mem::size_of::<ProgramHeader<T>>();
        Some(Program::parse(self.data, offset))
    }
    pub fn iter(&self) -> impl Iterator<Item = Result<Program<'a, T>, ParseError>> + 'a {
        let data = self.data;
        let offset = self.offset;
        let len = self.len;
        let mut index = 0usize;
        core::iter::from_fn(move || {
            if index >= len {
                return None;
            }
            let offset = offset + index * core::mem::size_of::<ProgramHeader<T>>();
            let ans = Program::parse(data, offset);
            index += 1;
            Some(ans)
        })
    }
}

pub struct Program<'a, T: Interpreter> {
    pheader: &'a ProgramHeader<T>,
    content: &'a [u8],
}

impl<'a, T: Interpreter> Program<'a, T> {
    pub(crate) fn parse(data: &'a [u8], offset: usize) -> Result<Self, ParseError> {
        use ParseError::*;
        use ProgramType::*;
        let pheader: &'a ProgramHeader<T> = read(data, offset).ok_or(BrokenHeader)?;
        let typa = pheader.checked_type().ok_or(BadProperty)?;
        match typa {
            Null => Ok(Program {
                pheader,
                content: &[],
            }),
            _ => {
                let content_offset = as_offset::<T>(pheader.offset()).ok_or(BrokenBody)?;
                let content_size = as_offset::<T>(pheader.filesz()).ok_or(BrokenBody)?;
                let content = read_n::<u8>(data, content_offset, content_size).ok_or(BrokenBody)?;
                Ok(Program { pheader, content })
            }
        }
    }
    pub fn header(&self) -> &'a ProgramHeader<T> {
        self.pheader
    }
    pub fn content(&self) -> &'a [u8] {
        self.content
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct ProgramHeader<T: Interpreter> {
    pub typa: U32,
    pub flags64: T::PropU32If64,
    pub offset: Usize<T>,
    pub vaddr: Usize<T>,
    pub paddr: Usize<T>,
    pub filesz: Usize<T>,
    pub memsz: Usize<T>,
    pub flags32: T::PropU32If32,
    pub align: Usize<T>,
}

impl<T: Interpreter> ProgramHeader<T> {
    pub fn checked_type(&self) -> Option<ProgramType> {
        ProgramType::try_from(T::interpret(self.typa)).ok()
    }
    /// # Panics
    ///
    /// Panics if it's not a vaild program type.
    pub fn typa(&self) -> ProgramType {
        self.checked_type().unwrap()
    }
    pub fn flags(&self) -> ProgramFlags {
        T::interpret((self.flags32, self.flags64)).into()
    }
    pub fn offset(&self) -> Integer<T> {
        T::interpret(self.offset)
    }
    pub fn vaddr(&self) -> Integer<T> {
        T::interpret(self.vaddr)
    }
    pub fn paddr(&self) -> Integer<T> {
        T::interpret(self.paddr)
    }
    pub fn filesz(&self) -> Integer<T> {
        T::interpret(self.filesz)
    }
    pub fn memsz(&self) -> Integer<T> {
        T::interpret(self.memsz)
    }
    pub fn align(&self) -> Integer<T> {
        T::interpret(self.align)
    }
}

unsafe impl<T: Interpreter> Pod for ProgramHeader<T> {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgramType {
    /// Program header table entry unused.
    Null,
    /// Loadable segment.
    Load,
    /// Dynamic linking information.
    Dynamic,
    /// Interpreter information.
    Interp,
    /// Auxiliary information.
    Note,
    /// Reserved.
    Shlib,
    /// Segment containing program header table itself.
    Phdr,
    /// Thread-Local Storage template.
    Tls,
    /// Operating system-specific.
    OsSpecific(u32),
    /// Processor-specific.
    ProcessorSpecific(u32),
}

impl TryFrom<u32> for ProgramType {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        use ProgramType::*;
        match value {
            0 => Ok(Null),
            1 => Ok(Load),
            2 => Ok(Dynamic),
            3 => Ok(Interp),
            4 => Ok(Note),
            5 => Ok(Shlib),
            6 => Ok(Phdr),
            7 => Ok(Tls),
            x @ 0x60000000..=0x6FFFFFFF => Ok(OsSpecific(x)),
            x @ 0x70000000..=0x7FFFFFFF => Ok(ProcessorSpecific(x)),
            _ => Err(()),
        }
    }
}

impl From<ProgramType> for u32 {
    fn from(value: ProgramType) -> Self {
        use ProgramType::*;
        match value {
            Null => 0,
            Load => 1,
            Dynamic => 2,
            Interp => 3,
            Note => 4,
            Shlib => 5,
            Phdr => 6,
            Tls => 7,
            OsSpecific(x) => x,
            ProcessorSpecific(x) => x,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, From, Into, BitXor, BitAnd, BitOr, LowerHex)]
pub struct ProgramFlags(pub u32);

impl ProgramFlags {
    /// Execute permission
    pub const EXECUTE: Self = Self(0x1);
    /// Writing permission
    pub const WRITE: Self = Self(0x2);
    /// Read permission
    pub const READ: Self = Self(0x4);
    /// OS specific mask
    pub const MASKOS: Self = Self(0x0ff00000);
    /// Processor specific mask
    pub const MASKPROCESSOR: Self = Self(0xf0000000);
}
