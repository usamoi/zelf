use crate::context::*;
use crate::utils::*;

#[derive(Debug, Clone)]
pub enum ParseCompressionError {
    BadHeader,
    BadPropertyType,
    BadContent,
}

/// Compressed section.
#[derive(Debug, Clone, Copy)]
pub struct Compression<'a, T: Context> {
    header: &'a CompressionHeader<T>,
    content: &'a [u8],
}

impl<'a, T: Context> Compression<'a, T> {
    pub fn parse(content: &'a [u8]) -> Result<Self, ParseCompressionError> {
        use ParseCompressionError::*;
        let mut offset = 0usize;
        let header: &CompressionHeader<T> = read(content, offset).ok_or(BadHeader)?;
        let _type = header.checked_type().ok_or(BadPropertyType)?;
        offset += core::mem::size_of::<CompressionHeader<T>>();
        let content = read_s(&content[offset..]).ok_or(BadContent)?;
        Ok(Self { header, content })
    }
    pub fn header(&self) -> &'a CompressionHeader<T> {
        self.header
    }
    pub fn content(&self) -> &'a [u8] {
        self.content
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct CompressionHeader<T: Context> {
    pub typa: PropU32,
    pub size: T::PropUsize,
    pub addralign: T::PropUsize,
}

impl<T: Context> CompressionHeader<T> {
    pub fn checked_type(&self) -> Option<CompressionType> {
        T::interpret(self.typa).try_into().ok()
    }
    pub fn typa(&self) -> CompressionType {
        self.checked_type().unwrap()
    }
    pub fn size(&self) -> T::Integer {
        T::interpret(self.size)
    }
    pub fn addralign(&self) -> T::Integer {
        T::interpret(self.addralign)
    }
}

unsafe impl<T: Context> Pod for CompressionHeader<T> {}

#[derive(Debug, Clone, Copy)]
pub enum CompressionType {
    Zlib,
    OsSpecific(u32),
    ProcessorSpecific(u32),
}

impl TryFrom<u32> for CompressionType {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        use CompressionType::*;
        match value {
            1 => Ok(Zlib),
            x @ 0x60000000..=0x6fffffff => Ok(OsSpecific(x)),
            x @ 0x70000000..=0x7fffffff => Ok(ProcessorSpecific(x)),
            _ => Err(()),
        }
    }
}

impl From<CompressionType> for u32 {
    fn from(value: CompressionType) -> u32 {
        use CompressionType::*;
        match value {
            Zlib => 1,
            OsSpecific(x) => x,
            ProcessorSpecific(x) => x,
        }
    }
}
