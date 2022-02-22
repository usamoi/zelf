use crate::utils::*;
use crate::ParseError;
use crate::{Class, Data, Version};

#[repr(C)]
#[derive(Debug, Clone)]
pub struct Ident {
    pub magic: [u8; 4],
    pub class: u8,
    pub data: u8,
    pub version: u8,
    pub osabi: u8,
    pub abiversion: u8,
    pub reserved: [u8; 7],
}

impl Ident {
    pub fn parse(data: &[u8]) -> Result<&Self, ParseError> {
        use ParseError::*;
        let ident: &Ident = read(data, 0).ok_or(BrokenHeader)?;
        if ident.magic != [0x7F, b'E', b'L', b'F'] {
            return Err(BadProperty);
        }
        let _class = ident.checked_class().ok_or(BadProperty)?;
        let _data = ident.checked_data().ok_or(BadProperty)?;
        let _version = ident.checked_version().ok_or(BadProperty)?;
        Ok(unsafe { &*(data.as_ptr() as *const Ident) })
    }
    pub fn magic(&self) -> [u8; 4] {
        self.magic
    }
    pub fn checked_class(&self) -> Option<Class> {
        Class::try_from(self.class).ok()
    }
    /// # Panics
    ///
    /// Panics if it's not a vaild ELF class.
    pub fn class(&self) -> Class {
        self.checked_class().unwrap()
    }
    pub fn checked_data(&self) -> Option<Data> {
        Data::try_from(self.data).ok()
    }
    /// # Panics
    ///
    /// Panics if it's not a vaild ELF data encoding.
    pub fn data(&self) -> Data {
        self.checked_data().unwrap()
    }
    pub fn checked_version(&self) -> Option<Version> {
        Version::try_from(self.version).ok()
    }
    /// # Panics
    ///
    /// Panics if it's not a vaild ELF version.
    pub fn version(&self) -> Version {
        self.checked_version().unwrap()
    }
    pub fn os_abi(&self) -> u8 {
        self.osabi
    }
    pub fn abi_version(&self) -> u8 {
        self.abiversion
    }
}

unsafe impl Pod for Ident {}

/// No extensions or unspecified
pub const IDENT_OSABI_NONE: u8 = 0;
/// Hewlett-Packard HP-UX
pub const IDENT_OSABI_HPUX: u8 = 1;
/// NetBSD
pub const IDENT_OSABI_NETBSD: u8 = 2;
/// GNU/Linux
pub const IDENT_OSABI_GNU: u8 = 3;
/// Sun Solaris
pub const IDENT_OSABI_SOLARIS: u8 = 6;
/// AIX
pub const IDENT_OSABI_AIX: u8 = 7;
/// IRIX
pub const IDENT_OSABI_IRIX: u8 = 8;
/// FreeBSD
pub const IDENT_OSABI_FREEBSD: u8 = 9;
/// Compaq TRU64 UNIX
pub const IDENT_OSABI_TRU64: u8 = 10;
/// Novell Modesto
pub const IDENT_OSABI_MODESTRO: u8 = 11;
/// Open BSD
pub const IDENT_OSABI_OPENBSD: u8 = 12;
/// Open VMS
pub const IDENT_OSABI_OPENVMS: u8 = 13;
/// Hewlett-Packard Non-Stop Kernel
pub const IDENT_OSABI_NSK: u8 = 14;
/// Amiga Research OS
pub const IDENT_OSABI_AROS: u8 = 15;
/// The FenixOS highly scalable multi-core OS
pub const IDENT_OSABI_FENIXOS: u8 = 16;
/// Nuxi CloudABI
pub const IDENT_OSABI_CLOUDABI: u8 = 17;
/// Stratus Technologies OpenVOS
pub const IDENT_OSABI_OPENVOS: u8 = 18;
