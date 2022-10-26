use ::core::{
    cmp::Ordering,
    fmt,
    hash::{Hash, Hasher},
};
use ::mischief::{In, Region, Slot};
use ::ptr_meta::Pointee;
use ::situ::{DropRaw, Mut, Val};

use crate::{Emplace, Move, Portable};

/// Alias for `i8`.
pub type I8 = i8;

/// Alias for `u8`.
pub type U8 = u8;

/// Alias for `bool`.
pub type Bool = bool;

/// Alias for `()`.
pub type Unit = ();

macro_rules! impl_primitive {
    (@base $portable:ty, $native:ty) => {
        impl From<$native> for $portable {
            #[inline]
            fn from(value: $native) -> Self {
                <$portable>::from_ne(value)
            }
        }

        impl From<$portable> for $native {
            #[inline]
            fn from(value: $portable) -> Self {
                value.to_ne()
            }
        }

        impl PartialEq for $portable
        where
            $native: PartialEq,
        {
            #[inline]
            fn eq(&self, other: &Self) -> bool {
                self.to_ne().eq(&other.to_ne())
            }
        }

        impl PartialOrd for $portable
        where
            $native: PartialOrd,
        {
            #[inline]
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                self.to_ne().partial_cmp(&other.to_ne())
            }
        }

        impl fmt::Debug for $portable
        where
            $native: fmt::Debug,
        {
            #[inline]
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                self.to_ne().fmt(f)
            }
        }

        impl fmt::Display for $portable
        where
            $native: fmt::Display,
        {
            #[inline]
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                self.to_ne().fmt(f)
            }
        }

        impl DropRaw for $portable {
            #[inline]
            unsafe fn drop_raw(_: Mut<'_, Self>) {}
        }

        // SAFETY:
        // - All primitives are `Sized` and always have metadata `()`, so
        //  `emplaced_meta` always returns valid metadata for them.
        // - `emplace_unsized_unchecked` initializes `out` by writing to it.
        unsafe impl<R: Region> Emplace<$portable, R> for $native {
            fn emplaced_meta(&self) -> <$portable as Pointee>::Metadata {}

            unsafe fn emplace_unsized_unchecked(
                self,
                out: In<Slot<'_, $portable>, R>,
            ) {
                In::into_inner(out).write(<$portable>::from_ne(self));
            }
        }

        // SAFETY: `move_unsized_unchecked` initializes its `out` parameter by
        // writing to it.
        unsafe impl<R: Region> Move<R> for $portable {
            unsafe fn move_unsized_unchecked(
                this: In<Val<'_, Self>, R>,
                out: In<Slot<'_, Self>, R>,
            ) {
                In::into_inner(out).write(Val::read(In::into_inner(this)));
            }
        }
    };
    ($portable:ty, $native:ty) => {
        impl_primitive!(@base $portable, $native);

        impl Eq for $portable
        where
            $native: Eq,
        {}

        impl Ord for $portable
        where
            $native: Ord,
        {
            #[inline]
            fn cmp(&self, other: &Self) -> Ordering {
                self.to_ne().cmp(&other.to_ne())
            }
        }

        impl Hash for $portable
        where
            $native: Hash,
        {
            #[inline]
            fn hash<H: Hasher>(&self, state: &mut H) {
                self.to_ne().hash(state)
            }
        }
    }
}

macro_rules! impl_multibyte_integer {
    ($portable:ident, $align:expr, $native:ty) => {
        #[doc = concat!("A portable `", stringify!($native), "`.")]
        #[derive(Clone, Copy)]
        #[repr(C, align($align))]
        pub struct $portable {
            value: $native,
        }

        // SAFETY: Multibyte integers have a well-defined size, an alignment
        // equal to their size, and their bit patterns are adjusted for
        // endianness.
        unsafe impl Portable for $portable {}

        impl $portable {
            #[doc = "Returns the `"]
            #[doc = stringify!($portable)]
            #[doc = "` corresponding to the given `"]
            #[doc = stringify!($native)]
            #[doc = "`."]
            #[inline]
            pub const fn from_ne(value: $native) -> Self {
                Self {
                    value: {
                        #[cfg(feature = "little_endian")]
                        {
                            value.to_le()
                        }
                        #[cfg(feature = "big_endian")]
                        {
                            value.to_be()
                        }
                    },
                }
            }

            #[doc = "Returns the `"]
            #[doc = stringify!($native)]
            #[doc = "` corresponding to this `"]
            #[doc = stringify!($portable)]
            #[doc = "`."]
            #[inline]
            pub const fn to_ne(self) -> $native {
                #[cfg(feature = "little_endian")]
                {
                    <$native>::from_le(self.value)
                }
                #[cfg(feature = "big_endian")]
                {
                    <$native>::from_be(self.value)
                }
            }
        }

        impl_primitive!($portable, $native);
    };
}

impl_multibyte_integer!(I16, 2, i16);
impl_multibyte_integer!(I32, 4, i32);
impl_multibyte_integer!(I64, 8, i64);
impl_multibyte_integer!(I128, 16, i128);
impl_multibyte_integer!(U16, 2, u16);
impl_multibyte_integer!(U32, 4, u32);
impl_multibyte_integer!(U64, 8, u64);
impl_multibyte_integer!(U128, 16, u128);

/// A portable `f32`.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct F32 {
    int_repr: U32,
}

// SAFETY: `U32` is `Portable` and `F32` is `repr(transparent)`, so `F32` has
// the same layout and bytewise representation guarantees as `U32`.
unsafe impl Portable for F32 where U32: Portable {}

impl F32 {
    /// Returns the `F32` corresponding to the given `f32`.
    #[inline]
    pub fn from_ne(value: f32) -> Self {
        Self {
            int_repr: U32::from_ne(value.to_bits()),
        }
    }

    /// Returns the `f32` corresponding to this `F32`.
    #[inline]
    pub fn to_ne(self) -> f32 {
        f32::from_bits(self.int_repr.to_ne())
    }
}

impl_primitive!(@base F32, f32);

/// A portable `f64`.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct F64 {
    int_repr: U64,
}

// SAFETY: `U64` is `Portable` and `F64` is `repr(transparent)`, so `F64` has
// the same layout and bytewise representation guarantees as `U64`.
unsafe impl Portable for F64 where U64: Portable {}

impl F64 {
    /// Returns the `F64` corresponding to the given `f64`.
    #[inline]
    pub fn from_ne(value: f64) -> Self {
        Self {
            int_repr: U64::from_ne(value.to_bits()),
        }
    }

    /// Returns the `f64` corresponding to this `F64`.
    #[inline]
    pub fn to_ne(self) -> f64 {
        f64::from_bits(self.int_repr.to_ne())
    }
}

impl_primitive!(@base F64, f64);

/// A portable `char`.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Char {
    int_repr: U32,
}

// SAFETY: `U32` is `Portable` and `Char` is `repr(transparent)` so `Char` has
// the same layout and bytewise representation guarantees as `U32`.
unsafe impl Portable for Char where U32: Portable {}

impl Char {
    /// Returns the `Char` corresponding to the given `char`.
    #[inline]
    pub fn from_ne(value: char) -> Self {
        Self {
            int_repr: U32::from_ne(u32::from(value)),
        }
    }

    /// Returns the `char` corresponding to this `Char`.
    #[inline]
    pub fn to_ne(self) -> char {
        // SAFETY: `int_repr` always contains a `u32` that is a valid `char`.
        unsafe { char::from_u32_unchecked(self.int_repr.to_ne()) }
    }
}

impl_primitive!(Char, char);
