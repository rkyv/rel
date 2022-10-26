//! A UTF-8 encoded, growable string.

use ::core::{fmt, ptr::copy_nonoverlapping};
use ::mischief::{In, Slot};
use ::munge::munge;
use ::ptr_meta::Pointee;
use ::rel_core::{Basis, DefaultBasis, Emplace, EmplaceExt, Move, Portable};
use ::situ::{
    alloc::RawRegionalAllocator,
    fmt::{DebugRaw, DisplayRaw},
    ops::{DerefMutRaw, DerefRaw},
    str::{from_raw_utf8_unchecked, from_raw_utf8_unchecked_mut},
    DropRaw,
    Mut,
    Ref,
};

use crate::{alloc::RelAllocator, vec, RelVec};

/// A relative counterpart to `String`.
#[derive(DropRaw, Move, Portable)]
#[repr(C)]
pub struct RelString<A: RawRegionalAllocator, B: Basis = DefaultBasis> {
    vec: RelVec<u8, A, B>,
}

impl<A: RawRegionalAllocator, B: Basis> RelString<A, B> {
    /// Returns a reference to the underlying allocator.
    #[inline]
    pub fn allocator(this: Ref<'_, Self>) -> Ref<'_, A> {
        munge!(let RelString { vec } = this);
        RelVec::allocator(vec)
    }

    /// Returns a bytes slice of this `RelString`'s contents.
    #[inline]
    pub fn as_bytes(this: Ref<'_, Self>) -> Ref<'_, [u8]> {
        munge!(let RelString { vec } = this);
        DerefRaw::deref_raw(vec)
    }

    /// Returns a string slice of the `RelString`'s contents.
    #[inline]
    pub fn as_str(this: Ref<'_, Self>) -> Ref<'_, str> {
        // SAFETY: The bytes of a `RelString` are always valid UTF-8.
        unsafe { from_raw_utf8_unchecked(Self::as_bytes(this)) }
    }

    /// Returns a mutable reference to the contents of this `RelString`.
    ///
    /// # Safety
    ///
    /// The returned `Mut<'_, RelVec<u8, A, B>>` allows writing bytes which are
    /// not valid UTF-8. If this constraint is violated, using the original
    /// `RelString` after dropping the `Mut` may violate memory safety, as other
    /// code may assume that `RelStrings` only contain valid UTF-8.
    #[inline]
    pub unsafe fn as_mut_vec(this: Mut<'_, Self>) -> Mut<'_, RelVec<u8, A, B>> {
        munge!(let RelString { vec } = this);
        vec
    }

    /// Returns a mutable string slice of the `RelString`'s contents.
    #[inline]
    pub fn as_mut_str(this: Mut<'_, Self>) -> Mut<'_, str> {
        // SAFETY: The contents of the `RelVec` are returned as a mutable `str`,
        // which cannot be mutated into invalid UTF-8.
        let vec = unsafe { Self::as_mut_vec(this) };
        let bytes = DerefMutRaw::deref_mut_raw(vec);
        // SAFETY: The bytes of a `RelString` are always valid UTF-8.
        unsafe { from_raw_utf8_unchecked_mut(bytes) }
    }

    /// Returns this `RelString`'s capacity, in bytes.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.vec.capacity()
    }

    /// Truncates this `RelString`, removing all contents.
    ///
    /// While this means the `String` will have a length of zero, it does not
    /// affect its capacity.
    #[inline]
    pub fn clear(this: Mut<'_, Self>) {
        munge!(let RelString { vec } = this);
        RelVec::clear(vec)
    }

    /// Returns the length of this `RelString`, in bytes, not `char`s or
    /// graphemes. In other words, it might not be what a human considers the
    /// length of the string.
    #[inline]
    pub fn len(&self) -> usize {
        self.vec.len()
    }

    /// Returns whether this `RelString` is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.vec.is_empty()
    }
}

impl<A: RawRegionalAllocator, B: Basis> DerefRaw for RelString<A, B> {
    type Target = str;

    fn deref_raw(this: Ref<'_, Self>) -> Ref<'_, Self::Target> {
        Self::as_str(this)
    }
}

/// An emplacer for a `RelString` that copies its bytes from a `str`.
pub struct Clone<'a, R>(pub R, pub &'a str);

// SAFETY:
// - `RelString` is `Sized` and always has metadata `()`, so `emplaced_meta`
//   always returns valid metadata for it.
// - `emplace_unsized_unchecked` initializes its `out` parameter by emplacing to
//   each field.
unsafe impl<A, B, R> Emplace<RelString<A, B>, R::Region> for Clone<'_, R>
where
    A: DropRaw + RawRegionalAllocator<Region = R::Region>,
    B: Basis,
    R: RelAllocator<A>,
{
    fn emplaced_meta(&self) -> <RelString<A, B> as Pointee>::Metadata {}

    unsafe fn emplace_unsized_unchecked(
        self,
        out: In<Slot<'_, RelString<A, B>>, A::Region>,
    ) {
        let len = self.1.len();

        munge!(let RelString { vec: out_vec } = out);
        let mut vec =
            In::into_inner(vec::WithCapacity(self.0, len).emplace_mut(out_vec));
        // SAFETY:
        // - `src.1.as_ptr()` is valid for reads of `len` bytes because it is a
        //   pointer to a `&str` of length `len`.
        // - `RelVec::as_mut_ptr` is valid for writes of `len` bytes because it
        //   was emplaced with capacity `len`.
        // - Both `str` and `RelVec<u8>` are allocated with the proper alignment
        //   for `u8`.
        // - The two regions of memory cannot overlap because `vec` is newly
        //   allocated and points to unaliased memory.
        unsafe {
            copy_nonoverlapping(
                self.1.as_ptr(),
                RelVec::as_mut_ptr(vec.as_mut()),
                len,
            );
        }
        // SAFETY:
        // - `len` is exactly equal to capacity.
        // - We initialized all of the bytes of the `RelVec` by copying the
        //   bytes of the emplaced string to them.
        unsafe {
            RelVec::set_len(vec, len);
        }
    }
}

impl<A: RawRegionalAllocator, B: Basis> DebugRaw for RelString<A, B> {
    fn fmt_raw(
        this: Ref<'_, Self>,
        f: &mut fmt::Formatter<'_>,
    ) -> Result<(), fmt::Error> {
        fmt::Debug::fmt(&*Self::as_str(this), f)
    }
}

impl<A: RawRegionalAllocator, B: Basis> DisplayRaw for RelString<A, B> {
    fn fmt_raw(
        this: Ref<'_, Self>,
        f: &mut fmt::Formatter<'_>,
    ) -> Result<(), fmt::Error> {
        fmt::Display::fmt(&*Self::as_str(this), f)
    }
}
