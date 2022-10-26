mod impls;

use ::mischief::{In, Region, Slot};
use ::ptr_meta::{metadata, Pointee};
pub use ::rel_core_derive::Move;
use ::situ::{DropRaw, Val};

/// An emplaced value that can be moved.
///
/// # Safety
///
/// `move_unsized_unchecked` must initialize its `out` parameter.
pub unsafe trait Move<R: Region>: DropRaw {
    /// Moves a value into a given slot within some memory region.
    ///
    /// # Safety
    ///
    /// `out` must have the same metadata as `this`.
    unsafe fn move_unsized_unchecked(
        this: In<Val<'_, Self>, R>,
        out: In<Slot<'_, Self>, R>,
    );
}

/// An extension trait for `Move` that provides a variety of convenient movement
/// methods.
///
/// # Safety
///
/// `move_unsized` and `r#move`/`move_` must initialize their `out` parameters.
pub unsafe trait MoveExt<R: Region>: Move<R> {
    /// Moves a value into a given slot within some memory region.
    ///
    /// # Panics
    ///
    /// Panics if `out` does not have the same metadata as `this`.
    fn move_unsized(this: In<Val<'_, Self>, R>, out: In<Slot<'_, Self>, R>);

    /// Moves a `Sized` value into a given slot within some memory region.
    fn r#move(this: In<Val<'_, Self>, R>, out: In<Slot<'_, Self>, R>)
    where
        Self: Sized;

    /// Moves a `Sized` value into a given slot within some memory region.
    ///
    /// This is an alternate name for [`move`](MoveExt::move), which uses a
    /// raw identifier in the name.
    fn move_(this: In<Val<'_, Self>, R>, out: In<Slot<'_, Self>, R>)
    where
        Self: Sized;
}

// SAFETY: `move_unsized` and `r#move`/`move_` initialize their `out` paramters
// by calling `move_unsized_unchecked`.
unsafe impl<T: Move<R> + Pointee + ?Sized, R: Region> MoveExt<R> for T {
    fn move_unsized(this: In<Val<'_, Self>, R>, out: In<Slot<'_, Self>, R>) {
        assert!(metadata(this.ptr().as_ptr()) == metadata(out.ptr().as_ptr()));
        // SAFETY: We have asserted that `out` has the same metadata as `this`.
        unsafe {
            Move::move_unsized_unchecked(this, out);
        }
    }

    fn r#move(this: In<Val<'_, Self>, R>, out: In<Slot<'_, Self>, R>)
    where
        Self: Sized,
    {
        // SAFETY: Because `Self: Sized`, its pointer metadata is `()` and so
        // the metadata of `this` and `out` must be the same because there is
        // only one possible value of `()`.
        unsafe {
            Move::move_unsized_unchecked(this, out);
        }
    }

    fn move_(this: In<Val<'_, Self>, R>, out: In<Slot<'_, Self>, R>)
    where
        Self: Sized,
    {
        Self::r#move(this, out);
    }
}
