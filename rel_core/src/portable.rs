use ::core::{
    cell::Cell,
    marker::{PhantomData, PhantomPinned},
    mem::MaybeUninit,
};
use ::mischief::{GhostRef, StaticToken};
pub use ::rel_core_derive::Portable;

/// A type that has the same representation on all targets.
///
/// # Safety
///
/// `Portable` types must have the same layout and bytewise representation on
/// all targets.
pub unsafe trait Portable {}

// Sources:
// https://doc.rust-lang.org/reference/types/boolean.html
// https://doc.rust-lang.org/reference/types/numeric.html
// https://doc.rust-lang.org/reference/type-layout.html

// SAFETY: `bool` has a size and alignment of 1 each. `false` has the bit
// pattern `0x00` and `true` has the bit pattern `0x01`.
unsafe impl Portable for bool {}

// SAFETY: `u8` has a size and alignment of 1 each. Values are represented as
// unsigned integers.
unsafe impl Portable for u8 {}

// SAFETY: `i8` has a size and alignment of 1 each. Values are represented as
// signed two's complement integers.
unsafe impl Portable for i8 {}

// SAFETY: `()` has a size of 0 and an alignment of 1. It has no bit patterns.
unsafe impl Portable for () {}

// SAFETY: `[T; N]` has a size of `size_of::<T>() * N`, an alignment of
// `align_of::<T>()`, and contains its `Portable` elements in order.
unsafe impl<T: Portable, const N: usize> Portable for [T; N] {}

// SAFETY: `[T]` has a size of `size_of::<T>() * len`, an alignment of
// `align_of::<T>()`, and contains its `Portable` elements in order.
unsafe impl<T: Portable> Portable for [T] {}

// SAFETY: `str` is `Portable` because `[u8]` is `Portable`.
unsafe impl Portable for str {}

// SAFETY: `PhantomData` has a size of 0 and an alignment of 1. It has no bit
// patterns.
unsafe impl<T: ?Sized> Portable for PhantomData<T> {}

// SAFETY: `PhantomPinned` has a size of 0 and an alignment of 1. It has no bit
// patterns.
unsafe impl Portable for PhantomPinned {}

// SAFETY: `MaybeUninit<T>` has the same size, alignment, and bit patterns as
// `T`.
unsafe impl<T: Portable> Portable for MaybeUninit<T> {}

// SAFETY: `Cell<T>` is `Portable` if `T` is `Portable` because it is
// `repr(transparent)`.
unsafe impl<T: Portable + ?Sized> Portable for Cell<T> {}

// SAFETY: `GhostRef` has a size of 0 and an alignment of 1. It has no bit
// patterns.
unsafe impl<T> Portable for GhostRef<T> {}

// SAFETY: `StaticToken` has a size of 0 and an alignment of 1. It has no bit
// patterns.
unsafe impl Portable for StaticToken<'_> {}
