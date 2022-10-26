//! Relative pointers and related types.

use ::core::{fmt, marker::PhantomData};
use ::mischief::{In, Region, Slot};
use ::munge::munge;
use ::ptr_meta::Pointee;
use ::situ::{
    fmt::{DebugRaw, DisplayRaw},
    DropRaw,
    Ref,
};

use crate::{
    Basis,
    BasisPointee,
    DefaultBasis,
    Emplace,
    EmplaceExt,
    Move,
    Portable,
    RelPtr,
};

/// A reference stored using a relative pointer.
#[repr(C)]
#[derive(DropRaw, Move, Portable)]
#[rel_core = "crate"]
pub struct RelRef<'a, T, R, B = DefaultBasis>
where
    T: BasisPointee<B> + ?Sized,
    R: Region,
    B: Basis,
{
    inner: RelPtr<T, R, B>,
    _phantom: PhantomData<&'a T>,
}

impl<'a, T: BasisPointee<B> + ?Sized, R: Region, B: Basis> RelRef<'a, T, R, B> {
    /// Returns a `Ref` to the underlying value.
    pub fn deref(this: Ref<'_, Self>) -> Ref<'a, T> {
        munge!(let RelRef { inner, .. } = this);
        // SAFETY: The `RelPtr` in a `RelRef` is always non-null, properly
        // aligned, and valid for reads. It also never aliases any mutable
        // references, and is guaranteed to point to an initialized value.
        unsafe { RelPtr::as_ref(inner) }
    }
}

// SAFETY:
// - `RelRef` is `Sized` and always has metadata `()`, so `emplaced_meta` always
//   returns valid metadata for it.
// - `emplace_unsized_unchecked` initializes its `out` parameter.
unsafe impl<'a, T, R, B> Emplace<RelRef<'a, T, R, B>, R> for In<Ref<'a, T>, R>
where
    T: BasisPointee<B> + ?Sized,
    R: Region,
    B: Basis,
{
    #[inline]
    fn emplaced_meta(&self) -> <RelRef<'a, T, B> as Pointee>::Metadata {}

    unsafe fn emplace_unsized_unchecked(
        self,
        out: In<Slot<'_, RelRef<'a, T, R, B>>, R>,
    ) {
        munge!(let RelRef { inner: out_inner, .. } = out);

        self.as_raw().emplace(out_inner);
    }
}

impl<'a, T, R, B> DebugRaw for RelRef<'a, T, R, B>
where
    T: BasisPointee<B> + DebugRaw + ?Sized,
    R: Region,
    B: Basis,
{
    fn fmt_raw(
        this: Ref<'_, Self>,
        f: &mut fmt::Formatter<'_>,
    ) -> Result<(), fmt::Error> {
        DebugRaw::fmt_raw(RelRef::deref(this), f)
    }
}

impl<'a, T, R, B> DisplayRaw for RelRef<'a, T, R, B>
where
    T: BasisPointee<B> + DisplayRaw + ?Sized,
    R: Region,
    B: Basis,
{
    fn fmt_raw(
        this: Ref<'_, Self>,
        f: &mut fmt::Formatter<'_>,
    ) -> Result<(), fmt::Error> {
        DisplayRaw::fmt_raw(RelRef::deref(this), f)
    }
}
