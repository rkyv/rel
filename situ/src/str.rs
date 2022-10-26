//! Utilities for the `str` primitive type.

use ::core::str;

use crate::{Mut, Ref};

/// Converts a slice of bytes to a string slice without checking that the string
/// contains valid UTF-8.
///
/// # Safety
///
/// The bytes passed in must be valid UTF-8.
pub unsafe fn from_raw_utf8_unchecked(v: Ref<'_, [u8]>) -> Ref<'_, str> {
    // SAFETY: The caller has guaranteed that the bytes of `v` are valid UTF-8.
    let ptr = v.as_ptr();
    let str_ptr =
        ::ptr_meta::from_raw_parts(ptr.cast(), ::ptr_meta::metadata(ptr));
    // SAFETY: `str_ptr` is the pointer from `v`, and so:
    // - Is non-null, properly aligned, and valid for reads.
    // - Does not alias alias any other mutable references for `'_`.
    // - Points to an initialized value.
    unsafe { Ref::new_unchecked(str_ptr) }
}

/// Converts a slice of bytes to a string slice.
pub fn from_raw_utf8(v: Ref<'_, [u8]>) -> Result<Ref<'_, str>, str::Utf8Error> {
    str::from_utf8(&v)?;
    // SAFETY: `from_utf8` has checked that the byte slice is valid UTF-8.
    Ok(unsafe { from_raw_utf8_unchecked(v) })
}

/// Converts a slice of bytes to a mutable string slice without checking that
/// the string contains valid UTF-8.
///
/// # Safety
///
/// The bytes passed in must be valid UTF-8.
pub unsafe fn from_raw_utf8_unchecked_mut(v: Mut<'_, [u8]>) -> Mut<'_, str> {
    // SAFETY: The caller has guaranteed that the bytes of `v` are valid UTF-8.
    let ptr = v.as_ptr();
    let str_ptr =
        ::ptr_meta::from_raw_parts_mut(ptr.cast(), ::ptr_meta::metadata(ptr));
    // SAFETY: `str_ptr` is the pointer from `v`, and so:
    // - Is non-null, properly aligned, and valid for reads.
    // - Does not alias alias any other accessible references for `'_`.
    // - Points to an initialized, immovable value.
    unsafe { Mut::new_unchecked(str_ptr) }
}

/// Converts a slice of bytes to a mutable string slice.
pub fn from_raw_utf8_mut(
    mut v: Mut<'_, [u8]>,
) -> Result<Mut<'_, str>, str::Utf8Error> {
    str::from_utf8_mut(&mut v)?;
    // SAFETY: `from_utf8_mut` has checked that the byte slice is valid UTF-8.
    Ok(unsafe { from_raw_utf8_unchecked_mut(v) })
}
