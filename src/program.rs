use crate::context::PropU32;
use crate::context::*;
use crate::elf::Elf;
use crate::utils::{as_offset, read, read_n, Pod};
use core::marker::PhantomData;

#[derive(Debug, Clone)]
pub enum ParseProgramsError {
    BadPropertyPhentsize,
    BadProgramHeaders,
    BadArray,
}

#[derive(Debug, Clone)]
pub enum ParseProgramError {
    BadHeader,
    BadPropertyType,
    BadContent,
    BadPhdr,
    BadTls,
}

#[derive(Debug, Clone, Copy)]
pub struct Programs<'a, T: Context> {
    data: &'a [u8],
    offset: usize,
    num: u16,
    _maker: PhantomData<T>,
}

impl<'a, T: Context> Programs<'a, T> {
    pub fn parse(elf: Elf<'a, T>) -> Result<Option<Programs<'a, T>>, ParseProgramsError> {
        use ParseProgramsError::*;
        let data = elf.data();
        let offset = as_offset::<T>(elf.header().phoff()).ok_or(BadProgramHeaders)?;
        if offset == 0 {
            return Ok(None);
        }
        if elf.header().phentsize() as usize != core::mem::size_of::<ProgramHeader<T>>() {
            return Err(BadPropertyPhentsize);
        }
        let num = elf.header().phnum();
        read_n::<ProgramHeader<T>>(data, offset, num as usize).ok_or(BadProgramHeaders)?;
        Ok(Some(Self {
            data,
            offset,
            num,
            _maker: PhantomData,
        }))
    }
    pub fn num(&self) -> u16 {
        self.num
    }
}

pub struct Program<'a, T: Context> {
    pheader: &'a ProgramHeader<T>,
    content: &'a [u8],
}

impl<'a, T: Context> Program<'a, T> {
    pub fn parse(programs: Programs<'a, T>, index: u16) -> Option<Result<Self, ParseProgramError>> {
        use ParseProgramError::*;
        use ProgramType::*;
        if index >= programs.num {
            return None;
        }
        let offset = programs.offset + index as usize * core::mem::size_of::<ProgramHeader<T>>();
        fn helper<'a, T: Context>(
            programs: Programs<'a, T>,
            offset: usize,
        ) -> Result<Program<'a, T>, ParseProgramError> {
            let pheader: &'a ProgramHeader<T> = read(programs.data, offset).ok_or(BadHeader)?;
            let typa = pheader.checked_type().ok_or(BadPropertyType)?;
            if let Null = typa {
                return Ok(Program {
                    pheader,
                    content: &[],
                });
            }
            let content_offset = as_offset::<T>(pheader.offset()).ok_or(BadContent)?;
            let content_size = as_offset::<T>(pheader.filesz()).ok_or(BadContent)?;
            let content =
                read_n::<u8>(programs.data, content_offset, content_size).ok_or(BadContent)?;
            match typa {
                Phdr => {
                    if content_offset != programs.offset {
                        return Err(BadPhdr);
                    }
                    if content_size
                        != programs.num() as usize * core::mem::size_of::<ProgramHeader<T>>()
                    {
                        return Err(BadPhdr);
                    }
                }
                Tls => {
                    if pheader.flags() != ProgramFlags::READ {
                        return Err(BadTls);
                    }
                }
                _ => (),
            }
            Ok(Program { pheader, content })
        }
        Some(helper(programs, offset))
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
pub struct ProgramHeader<T: Context> {
    pub typa: PropU32,
    pub flags64: T::PropU32If64,
    pub offset: T::PropUsize,
    pub vaddr: T::PropUsize,
    pub paddr: T::PropUsize,
    pub filesz: T::PropUsize,
    pub memsz: T::PropUsize,
    pub flags32: T::PropU32If32,
    pub align: T::PropUsize,
}

impl<T: Context> ProgramHeader<T> {
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
    pub fn offset(&self) -> T::Integer {
        T::interpret(self.offset)
    }
    pub fn vaddr(&self) -> T::Integer {
        T::interpret(self.vaddr)
    }
    pub fn paddr(&self) -> T::Integer {
        T::interpret(self.paddr)
    }
    pub fn filesz(&self) -> T::Integer {
        T::interpret(self.filesz)
    }
    pub fn memsz(&self) -> T::Integer {
        T::interpret(self.memsz)
    }
    pub fn align(&self) -> T::Integer {
        T::interpret(self.align)
    }
}

unsafe impl<T: Context> Pod for ProgramHeader<T> {}

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
