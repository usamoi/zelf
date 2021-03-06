use crate::context::Context;
use core::fmt::Debug;

pub trait SealedContext {
    type PropUsize: Debug + Copy;

    type PropU32If32: Debug + Copy;

    type PropU32If64: Debug + Copy;

    type PropUsizeIf32: Debug + Copy;

    type PropUsizeIf64: Debug + Copy;
}

pub trait Interpret<T> {
    type Target;
    fn interpret(x: T) -> Self::Target;
}

pub fn as_offset<T: Context>(x: T::Integer) -> Option<usize> {
    let y: u64 = x.into();
    y.try_into().ok()
}

/// A marker trait indicating that a type is "Plain Old Data".
///
/// # Safety
///
/// A type can be "Plain Old Data" if it is C representation.
pub unsafe trait Pod: Sized {}

unsafe impl Pod for u8 {}

unsafe impl<const N: usize> Pod for [u8; N] {}

pub fn read<T: Pod>(data: &[u8], offset: usize) -> Option<&T> {
    if data.len() < offset.checked_add(core::mem::size_of::<T>())? {
        return None;
    }
    let p = (data.as_ptr() as usize).wrapping_add(offset) as *const T;
    Some(unsafe { &*p })
}

pub fn read_n<T: Pod>(data: &[u8], offset: usize, n: usize) -> Option<&[T]> {
    if data.len() < offset.checked_add(core::mem::size_of::<T>().checked_mul(n)?)? {
        return None;
    }
    let p = (data.as_ptr() as usize).wrapping_add(offset) as *const T;
    Some(unsafe { core::slice::from_raw_parts(p, n) })
}

pub fn read_s<T: Pod>(data: &[u8]) -> Option<&[T]> {
    if core::mem::size_of::<T>() == 0 {
        return None;
    }
    let p = data.as_ptr() as *const T;
    let n = data.len() / core::mem::size_of::<T>();
    Some(unsafe { core::slice::from_raw_parts(p, n) })
}

// todo: overflow checking
pub fn align(offset: usize, align: usize) -> usize {
    assert!(align.is_power_of_two());
    (offset.wrapping_sub(1) | align.wrapping_sub(1)).wrapping_add(1)
}

pub fn terminate(s: &[u8]) -> Option<&[u8]> {
    for (i, c) in s.iter().enumerate() {
        if *c == 0 {
            return Some(&s[..i]);
        }
    }
    None
}
