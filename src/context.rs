use crate::dynamic::{DynamicFlags32, DynamicFlags64};
use crate::section::{SectionFlags32, SectionFlags64};
use crate::utils::{Interpret, SealedContext};
use crate::{Class, Data, Version};
use core::fmt::Debug;

pub type PropU16 = [u8; 2];

pub type PropU32 = [u8; 4];

pub type PropU64 = [u8; 8];

pub trait Context: Copy + SealedContext + 'static
where
    Self: Interpret<Self::PropUsize, Target = Self::Integer>,
    Self: Interpret<(Self::PropU32If32, Self::PropU32If64), Target = u32>,
    Self: Interpret<(Self::PropUsizeIf32, Self::PropUsizeIf64), Target = Self::Integer>,
{
    const CLASS: Class;

    const DATA: Data;

    const VERSION: Version;

    type Integer: Copy + Debug + Ord + From<u32> + Into<u64>;

    type SectionFlags: Copy + Debug + From<Self::Integer> + Into<Self::Integer>;

    type DynamicFlags: Copy + Debug + From<Self::Integer> + Into<Self::Integer>;
}

impl<T: Context> Interpret<PropU16> for T {
    type Target = u16;
    fn interpret(x: PropU16) -> u16 {
        use Data::*;
        match T::DATA {
            Little => u16::from_le_bytes(x),
            Big => u16::from_be_bytes(x),
        }
    }
}

impl<T: Context> Interpret<PropU32> for T {
    type Target = u32;
    fn interpret(x: PropU32) -> u32 {
        use Data::*;
        match T::DATA {
            Little => u32::from_le_bytes(x),
            Big => u32::from_be_bytes(x),
        }
    }
}

impl<T: Context> Interpret<PropU64> for T {
    type Target = u64;
    fn interpret(x: PropU64) -> u64 {
        use Data::*;
        match T::DATA {
            Little => u64::from_le_bytes(x),
            Big => u64::from_be_bytes(x),
        }
    }
}

macro_rules! impl_context {
    ($t: ty, Class32, $data: ident, $version: ident) => {
        impl SealedContext for $t {
            type PropUsize = PropU32;

            type PropU32If32 = PropU32;

            type PropU32If64 = ();

            type PropUsizeIf32 = PropU32;

            type PropUsizeIf64 = ();
        }

        impl Context for $t {
            const CLASS: Class = Class::Class32;

            const DATA: Data = Data::$data;

            const VERSION: Version = Version::$version;

            type Integer = u32;

            type SectionFlags = SectionFlags32;

            type DynamicFlags = DynamicFlags32;
        }

        // todo: use specialization to replace it.

        impl<T, U> Interpret<(T, U)> for $t
        where
            Self: Interpret<T>,
        {
            type Target = <Self as Interpret<T>>::Target;

            fn interpret((x, _): (T, U)) -> Self::Target {
                Self::interpret(x)
            }
        }
    };
    ($t: ty, Class64, $data: ident, $version: ident) => {
        impl SealedContext for $t {
            type PropUsize = PropU64;

            type PropU32If32 = ();

            type PropU32If64 = PropU32;

            type PropUsizeIf32 = ();

            type PropUsizeIf64 = PropU64;
        }

        impl Context for $t {
            const CLASS: Class = Class::Class64;

            const DATA: Data = Data::$data;

            const VERSION: Version = Version::$version;

            type Integer = u64;

            type SectionFlags = SectionFlags64;

            type DynamicFlags = DynamicFlags64;
        }

        impl<T, U> Interpret<(T, U)> for $t
        where
            Self: Interpret<U>,
        {
            type Target = <Self as Interpret<U>>::Target;

            fn interpret((_, x): (T, U)) -> Self::Target {
                Self::interpret(x)
            }
        }
    };
}

/// Little endian, 32 bit, version 1.
#[derive(Debug, Clone, Copy)]
pub enum Little32 {}

impl_context!(Little32, Class32, Little, One);

/// Little endian, 64 bit, version 1.
#[derive(Debug, Clone, Copy)]
pub enum Little64 {}

impl_context!(Little64, Class64, Little, One);

/// Big endian, 32 bit, version 1.
#[derive(Debug, Clone, Copy)]
pub enum Big32 {}

impl_context!(Big32, Class32, Big, One);

/// Big endian, 64 bit, version 1.
#[derive(Debug, Clone, Copy)]
pub enum Big64 {}

impl_context!(Big64, Class64, Big, One);
