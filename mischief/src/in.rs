use ::core::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};
use ::munge::{Destructure, Restructure};
use ::ptr_meta::Pointee;

use crate::{Frame, Metadata, Pointer, Region, RegionalAllocator, Slot};

/// A pointer which has its pointee in a specific memory region.
#[derive(Clone, Copy)]
pub struct In<P, R: Region> {
    ptr: P,
    region: PhantomData<R>,
}

impl<P: Pointer, R: Region> In<P, R>
where
    P::Target: Pointee,
{
    /// Creates a new `In` from a pointer.
    pub fn new(ptr: P) -> Self
    where
        P: Within<R>,
    {
        // SAFETY: The pointee of `ptr` is must be located in `R` because `P`
        // implements `Within<R>`.
        unsafe { Self::new_unchecked(ptr) }
    }

    /// Creates a new `In` from a pointer.
    ///
    /// # Safety
    ///
    /// The pointee of `ptr` must be contained in `R`.
    pub unsafe fn new_unchecked(ptr: P) -> Self {
        Self {
            ptr,
            region: PhantomData,
        }
    }

    /// Maps this `In` to another pointer in its region.
    pub fn map<F, Q>(self, f: F) -> In<Q, R>
    where
        F: FnOnce(P) -> Q,
        Q: Pointer + Within<R>,
        Q::Target: Pointee,
    {
        In::new(f(Self::into_inner(self)))
    }

    /// Maps this `In` to another pointer in its region.
    ///
    /// # Safety
    ///
    /// The pointer returned by `f` must be completely contained in `R`.
    pub unsafe fn map_unchecked<F, Q>(self, f: F) -> In<Q, R>
    where
        F: FnOnce(P) -> Q,
        Q: Pointer,
        Q::Target: Pointee,
    {
        let ptr = Self::into_inner(self);
        // SAFETY: The caller has gauranteed that `R` completely contains the
        // mapped pointer.
        unsafe { In::new_unchecked(f(ptr)) }
    }

    /// Gets a raw `In` from this pointer.
    pub fn as_raw(&self) -> In<*mut P::Target, R> {
        // SAFETY: `self.ptr.deref_raw()` returns a pointer located in `R`.
        // Calling `deref_raw` on that returned `*mut P::Target` returns the
        // same pointer again because raw pointers `deref_raw` to themselves. So
        // `self.ptr.deref_raw()` also returns a pointer in `R` when `deref_raw`
        // is called on it.
        unsafe { In::new_unchecked(self.ptr.target()) }
    }
}

impl<P, R: Region> In<P, R> {
    /// Unwraps an `In`, returning the underlying pointer.
    pub fn into_inner(this: Self) -> P {
        this.ptr
    }

    /// Returns a reference to the pointer of this `In`.
    pub fn ptr(&self) -> &P {
        &self.ptr
    }

    /// Returns a mutable reference to the pointer of this `In`.
    ///
    /// # Safety
    ///
    /// The pointer must not be mutated to point outside of `R`.
    pub unsafe fn ptr_mut(&mut self) -> &mut P {
        &mut self.ptr
    }
}

impl<T: Pointee + ?Sized, A: RegionalAllocator> In<Frame<T, A>, A::Region>
where
    <T as Pointee>::Metadata: Metadata<T>,
{
    /// Returns a [`Slot`] of the internal contents.
    pub fn slot(&'_ mut self) -> In<Slot<'_, T>, A::Region> {
        // SAFETY: `slot` is wrapped in an `In` before being returned to prevent
        // it from being pointed outside of `R`, and we don't mutate it to point
        // elsewhere.
        let slot = unsafe { self.ptr_mut().slot() };
        // SAFETY: `slot` points to the same location as `self`, which points to
        // a slot located in `R`. Therefore, `slot` must also point to a a slot
        // located in `R`.
        unsafe { In::new_unchecked(slot) }
    }
}

impl<'a, T: Pointee + ?Sized, R: Region> In<Slot<'a, T>, R> {
    /// Gets a mutable borrow from this slot.
    pub fn as_mut<'b>(&'b mut self) -> In<Slot<'b, T>, R>
    where
        'a: 'b,
    {
        // SAFETY: The original and reborrowed slots are guaranteed to both
        // point to the same memory. Since the original slot is guaranteed to be
        // located in `R`, the reborrowed slot must also be located in `R`.
        unsafe { In::new_unchecked(self.ptr.as_mut()) }
    }
}

impl<P: Deref, R: Region> Deref for In<P, R> {
    type Target = P::Target;

    fn deref(&self) -> &Self::Target {
        self.ptr().deref()
    }
}

impl<P: DerefMut, R: Region> DerefMut for In<P, R> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.ptr.deref_mut()
    }
}

/// A `Pointer` that may be restructured with `munge`.
///
/// # Safety
///
/// This type's `Destructure::underlying` implementation must return the same
/// pointer as `Pointer::target`.
pub unsafe trait RestructurablePointer: Destructure {}

// SAFETY:
// - `In<P, R>` is destructured in the same way as its inner pointer is, so its
//   `Destructuring` type is `P::Underlying`.
// - `underlying` returns the underlying of the inner pointer, which is also
//   guaranteed to be non-null, properly-aligned, and valid for reads.
unsafe impl<P: Destructure, R: Region> Destructure for In<P, R> {
    type Underlying = P::Underlying;
    type Destructuring = P::Destructuring;

    fn underlying(&mut self) -> *mut Self::Underlying {
        self.ptr.underlying()
    }
}

type RsTarget<F, T> = <<F as Restructure<T>>::Restructured as Pointer>::Target;

// SAFETY: `restructure` returns a valid `In` created by restructuring using the
// inner pointer's restructuring. Because the inner pointer upholds its
// destructuring invariants, `In` does too.
unsafe impl<P, R: Region, U> Restructure<U> for In<P, R>
where
    P: RestructurablePointer + Restructure<U>,
    <P as Restructure<U>>::Restructured: Pointer,
    RsTarget<P, U>: Pointee,
{
    type Restructured = In<<P as Restructure<U>>::Restructured, R>;

    unsafe fn restructure(&self, ptr: *mut U) -> Self::Restructured {
        // SAFETY: The caller has guaranteed that `ptr` is a properly aligned
        // pointer to a subfield of the pointer underlying `self`, which is the
        // pointer underlying `self.ptr`.
        let restructured = unsafe { self.ptr.restructure(ptr) };
        // SAFETY: The restructured pointer must be a subfield of the original
        // pointer's pointee, so it's necessarily located in the same region as
        // the original as well. The safety requirements for
        // `RestructurablePointer` require that the pointer returned from
        // `Destructure::underlying` must be the same as the pointer returned
        // from `deref_raw`, so it's not possible to return different pointers
        // from each.
        unsafe { In::new_unchecked(restructured) }
    }
}

/// A pointer that knows where its target is located
///
/// # Safety
///
/// The `target` of the type must be located in `R`.
pub unsafe trait Within<R: Region>: Pointer {}
