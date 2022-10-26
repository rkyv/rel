mod impls;

use ::mischief::{In, Region, Slot};
use ::ptr_meta::{metadata, Pointee};
use ::situ::{DropRaw, Mut, Val};

/// A value emplacer.
///
/// # Safety
///
/// - `emplaced_meta` must return valid metadata for the value emplaced with
///   `emplace_unsized_unchecked`.
/// - `emplace_unsized_unchecked` must initialize its `out` parameter.
pub unsafe trait Emplace<T: DropRaw + Pointee + ?Sized, R: Region> {
    /// Returns the metadata of the `T` that this emplaces.
    ///
    /// For sized `T`, this is always `()`.
    fn emplaced_meta(&self) -> <T as Pointee>::Metadata;

    /// Emplaces a value into a given slot within some memory region.
    ///
    /// # Safety
    ///
    /// `out` must have the metadata returned by `emplaced_meta`.
    unsafe fn emplace_unsized_unchecked(self, out: In<Slot<'_, T>, R>);
}

/// An extension trait for `Emplace` that provides a variety of convenient
/// emplacement methods.
///
/// # Safety
///
/// `emplace_val_unsized` and `emplace_val` must initialize `out` and return it
/// as a `Val`.
pub unsafe trait EmplaceExt<T, R: Region>: Emplace<T, R>
where
    T: DropRaw + Pointee + ?Sized,
{
    /// Emplaces a value into a given slot within some memory region.
    ///
    /// # Panics
    ///
    /// Panics if `out` does not have the metadata returned by `emplaced_meta`.
    fn emplace_unsized(self, out: In<Slot<'_, T>, R>);

    /// Emplaces a sized value into a given slot within some memory region.
    ///
    /// This simply wraps a call to `emplace_unsized_unchecked`. Because `T` is
    /// `Sized`, the metadata of the slot's pointer must always match the
    /// metadata returned from `emplaced_meta`, and so it is safe.
    fn emplace(self, out: In<Slot<'_, T>, R>)
    where
        T: Sized;

    /// Emplaces a value into a given slot within some memory region and returns
    /// a mutable reference.
    ///
    /// # Safety
    ///
    /// `out` must have the metadata returned by `emplaced_meta`.
    unsafe fn emplace_mut_unsized(
        self,
        out: In<Slot<'_, T>, R>,
    ) -> In<Mut<'_, T>, R>;

    /// Emplaces a sized value into a given slot within some memory region and
    /// returns a mutable reference.
    fn emplace_mut(self, out: In<Slot<'_, T>, R>) -> In<Mut<'_, T>, R>;

    /// Emplaces a value into a given slot within some memory region and returns
    /// an initialized value.
    ///
    /// # Safety
    ///
    /// `out` must have the metadata returned by `emplaced_meta`.
    #[must_use]
    unsafe fn emplace_val_unsized(
        self,
        out: In<Slot<'_, T>, R>,
    ) -> In<Val<'_, T>, R>;

    /// Emplaces a sized value into a given slot within some memory region and
    /// returns an initialized value.
    ///
    /// This simply wraps a call to `emplace_val_unsized`. Because `T` is sized,
    /// the metadata of the slot's pointer must always match the metadata
    /// returned from `emplaced_meta`, and so it is safe.
    #[must_use]
    fn emplace_val(self, out: In<Slot<'_, T>, R>) -> In<Val<'_, T>, R>
    where
        T: Sized;
}

// SAFETY: `emplace_val` initializes `out` and returns it as a `Val`.
unsafe impl<E, T, R> EmplaceExt<T, R> for E
where
    E: Emplace<T, R>,
    T: DropRaw + Pointee + ?Sized,
    R: Region,
{
    fn emplace_unsized(self, out: In<Slot<'_, T>, R>) {
        assert!(self.emplaced_meta() == metadata::<T>(out.ptr().as_ptr()));
        // SAFETY: We have asserted that the metadata of `self` and `out` are
        // equal.
        unsafe {
            self.emplace_unsized_unchecked(out);
        }
    }

    #[inline]
    fn emplace(self, out: In<Slot<'_, T>, R>)
    where
        T: Sized,
    {
        // SAFETY: `out` can only have the metadata `()` and so it must be the
        // same as the metadata returned from `emplaced_meta`.
        unsafe { self.emplace_unsized_unchecked(out) }
    }

    unsafe fn emplace_mut_unsized(
        self,
        out: In<Slot<'_, T>, R>,
    ) -> In<Mut<'_, T>, R> {
        // SAFETY: The caller has guaranteed that `out` has the metadata
        // returned by `emplaced_meta`.
        let val = unsafe { self.emplace_val_unsized(out) };
        // SAFETY: `Val::leak` returns a mutable reference to its contained
        // value. Since the value is allocated in `R`, its contained value must
        // also be.
        unsafe { In::map_unchecked(val, Val::leak) }
    }

    #[inline]
    fn emplace_mut(self, out: In<Slot<'_, T>, R>) -> In<Mut<'_, T>, R> {
        // SAFETY: `out` can only have the metadata `()` and so it must be the
        // same as the metadata returned from `emplaced_meta`.
        unsafe { self.emplace_mut_unsized(out) }
    }

    #[must_use]
    unsafe fn emplace_val_unsized(
        self,
        mut out: In<Slot<'_, T>, R>,
    ) -> In<Val<'_, T>, R> {
        // SAFETY: The caller has guaranteed that `out` has the metadata
        // returned by `emplaced_meta`.
        unsafe {
            self.emplace_unsized_unchecked(out.as_mut());
        }
        // SAFETY: `emplace` is guaranteed to initialize `out`.
        let initialize = |s| unsafe { Val::from_slot_unchecked(s) };
        // SAFETY: the `Val` created from `out` points to the same memory and so
        // must be located in the same region.
        unsafe { out.map_unchecked(initialize) }
    }

    #[inline]
    #[must_use]
    fn emplace_val(self, out: In<Slot<'_, T>, R>) -> In<Val<'_, T>, R>
    where
        T: Sized,
    {
        // SAFETY: `out` can only have the metadata `()` and so it must be the
        // same as the metadata returned from `emplaced_meta`.
        unsafe { self.emplace_val_unsized(out) }
    }
}
