//! Basic functions for dealing with relative values in memory.
//!
//! This module contains functions for initializing and manipulating relative
//! values in memory.

use ::mischief::{In, Region};
use ::ptr_meta::{metadata, Pointee};
use ::situ::{DropRaw, Mut, Val};

use crate::Emplace;

/// Replaces the unsized value in `dest` by emplacing `src` into it.
///
/// # Safety
///
/// The caller must guarantee that `dest` must have the metadata returned from
/// `src.emplaced_meta`.
pub unsafe fn replace_unsized_unchecked<T, R, E>(
    dest: In<Mut<'_, T>, R>,
    src: E,
) where
    T: DropRaw + Pointee + ?Sized,
    R: Region,
    E: Emplace<T, R>,
{
    // SAFETY:
    // - We are taking ownership of the value in `dest`, so it is valid for
    //   dropping.
    // - The `Val` is dropped and emplaced over, so nothing can access the `Mut`
    //   between dropping and emplacement.
    let drop_into_slot = |m| unsafe { Val::drop(Mut::take(m)) };
    // SAFETY: `drop_into_slot` returns a slot of the same location as the given
    // `Mut`, so it must be located in the same region.
    let dest = unsafe { In::map_unchecked(dest, drop_into_slot) };
    // SAFETY: The caller has guaranteed that `dest` has the metadata returned
    // by `emplaced_meta`.
    unsafe {
        src.emplace_unsized_unchecked(dest);
    }
}

/// Replaces the unsized value in `dest` by emplacing `src` into it.
pub fn replace_unsized<T, R, E>(dest: In<Mut<'_, T>, R>, src: E)
where
    T: DropRaw + Pointee + ?Sized,
    R: Region,
    E: Emplace<T, R>,
{
    assert!(metadata(dest.ptr().as_ptr()) == src.emplaced_meta());
    // SAFETY: We have asserted that the `dest` has the metadata returned from
    // `src.emplaced_meta`.
    unsafe {
        replace_unsized_unchecked(dest, src);
    }
}

/// Replaces the value in `dest` by emplacing `src` into it.
pub fn replace<T, R, E>(dest: In<Mut<'_, T>, R>, src: E)
where
    T: DropRaw,
    R: Region,
    E: Emplace<T, R>,
{
    // SAFETY: `T` is sized, so `dest` must have the metadata returned from
    // `src.emplaced_meta`.
    unsafe {
        replace_unsized_unchecked(dest, src);
    }
}
