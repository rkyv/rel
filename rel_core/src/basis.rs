use ::core::{convert::Infallible, fmt::Debug, hash::Hash};
use ::ptr_meta::Pointee;
use ::situ::DropRaw;

/// A selection of types to use in place of `isize` and `usize`.
pub trait Basis {
    /// The type to use in place of `isize`.
    type Isize: Copy + DropRaw + Send + Sync + Ord + Hash + Unpin;
    /// The type to use in place of `usize`.
    type Usize: Copy + DropRaw + Send + Sync + Ord + Hash + Unpin;
    /// An error occurred during type conversion from a native `isize` or
    /// `usize`.
    type FromNativeError: Debug;
    /// An error occurred during type conversion to a native `isize` or `usize`.
    type ToNativeError: Debug;

    /// Returns the `Isize` corresponding to the given `isize`, or `Err` if the
    /// conversion fails.
    fn from_native_isize(
        value: isize,
    ) -> Result<Self::Isize, Self::FromNativeError>;

    /// Returns the `isize` corresponding to the given `Isize`, or `Err` if the
    /// conversion fails.
    fn to_native_isize(
        value: Self::Isize,
    ) -> Result<isize, Self::ToNativeError>;

    /// Returns the `Usize` corresponding to the given `usize`, or `Err` if the
    /// conversion fails.
    fn from_native_usize(
        value: usize,
    ) -> Result<Self::Usize, Self::FromNativeError>;

    /// Returns the `usize` corresponding to the given `Usize`, or `Err` if the
    /// conversion fails.
    fn to_native_usize(
        value: Self::Usize,
    ) -> Result<usize, Self::ToNativeError>;
}

/// A `Pointee` with metadata that changes with a choice of basis.
pub trait BasisPointee<B: Basis>: Pointee {
    /// The metadata associated with the pointee in the chosen basis.
    type BasisMetadata: Copy + Send + Sync + Ord + Hash + Unpin;
    /// An error occurred while converting metadata from the native
    /// representation.
    type FromNativeError: Debug;
    /// An error occurred while converting metadata to the native
    /// representation.
    type ToNativeError: Debug;

    /// Returns the pointer metadata in `B` corresponding to the given native
    /// pointer metadata, or `Err` if the conversion failed.
    fn from_native_metadata(
        metadata: Self::Metadata,
    ) -> Result<Self::BasisMetadata, Self::FromNativeError>;

    /// Returns the native pointer metadata corresponding to the given pointer
    /// metadata in `B`, or `Err` if the conversion failed.
    fn to_native_metadata(
        metadata: Self::BasisMetadata,
    ) -> Result<Self::Metadata, Self::ToNativeError>;
}

impl<T, B: Basis> BasisPointee<B> for T {
    type BasisMetadata = ();
    type FromNativeError = Infallible;
    type ToNativeError = Infallible;

    #[inline]
    fn from_native_metadata(
        _: Self::Metadata,
    ) -> Result<Self::BasisMetadata, Self::FromNativeError> {
        Ok(())
    }

    #[inline]
    fn to_native_metadata(
        _: Self::BasisMetadata,
    ) -> Result<Self::Metadata, Self::ToNativeError> {
        Ok(())
    }
}

impl<T, B: Basis> BasisPointee<B> for [T] {
    type BasisMetadata = B::Usize;
    type FromNativeError = B::FromNativeError;
    type ToNativeError = B::ToNativeError;

    #[inline]
    fn from_native_metadata(
        metadata: Self::Metadata,
    ) -> Result<Self::BasisMetadata, Self::FromNativeError> {
        B::from_native_usize(metadata)
    }

    #[inline]
    fn to_native_metadata(
        metadata: Self::BasisMetadata,
    ) -> Result<Self::Metadata, Self::ToNativeError> {
        B::to_native_usize(metadata)
    }
}

impl<B: Basis> BasisPointee<B> for str {
    type BasisMetadata = B::Usize;
    type FromNativeError = B::FromNativeError;
    type ToNativeError = B::ToNativeError;

    #[inline]
    fn from_native_metadata(
        metadata: Self::Metadata,
    ) -> Result<Self::BasisMetadata, Self::FromNativeError> {
        B::from_native_usize(metadata)
    }

    #[inline]
    fn to_native_metadata(
        metadata: Self::BasisMetadata,
    ) -> Result<Self::Metadata, Self::ToNativeError> {
        B::to_native_usize(metadata)
    }
}

#[cfg(feature = "basis_16")]
macro_rules! choose_basis {
    ($b16:ty, $b32:ty, $b64:ty) => {
        $b16
    };
}

#[cfg(feature = "basis_32")]
macro_rules! choose_basis {
    ($b16:ty, $b32:ty, $b64:ty) => {
        $b32
    };
}

#[cfg(feature = "basis_64")]
macro_rules! choose_basis {
    ($b16:ty, $b32:ty, $b64:ty) => {
        $b64
    };
}

#[cfg(any(
    all(target_pointer_width = "32", feature = "basis_16"),
    all(target_pointer_width = "64", not(feature = "basis_64")),
))]
macro_rules! compare_basis_to_pointer_width {
    (
        lt { $($lts:item)* }
        le { $($les:item)* }
        ge { $($ges:item)* }
        gt { $($gts:item)* }
    ) => {
        $($lts)* $($les)*
    }
}

#[cfg(any(
    all(target_pointer_width = "16", feature = "basis_16"),
    all(target_pointer_width = "32", feature = "basis_32"),
    all(target_pointer_width = "64", feature = "basis_64"),
))]
macro_rules! compare_basis_to_pointer_width {
    (
        lt { $($lts:item)* }
        le { $($les:item)* }
        ge { $($ges:item)* }
        gt { $($gts:item)* }
    ) => {
        $($($les)*)? $($($ges)*)?
    }
}

#[cfg(any(
    all(target_pointer_width = "16", not(feature = "basis_16")),
    all(target_pointer_width = "32", feature = "basis_64"),
))]
macro_rules! compare_basis_to_pointer_width {
    (
        lt { $($lts:item)* }
        le { $($les:item)* }
        ge { $($ges:item)* }
        gt { $($gts:item)* }
    ) => {
        $($ges)* $($gts)*
    }
}

/// The default [`Basis`].
///
/// The settings this basis uses will change depending on the enabled feature
/// flags.
pub struct DefaultBasis;

impl Basis for DefaultBasis {
    type Isize = choose_basis!(
        crate::primitive::I16,
        crate::primitive::I32,
        crate::primitive::I64
    );
    type Usize = choose_basis!(
        crate::primitive::U16,
        crate::primitive::U32,
        crate::primitive::U64
    );

    compare_basis_to_pointer_width! {
        lt {
            type FromNativeError = ::core::num::TryFromIntError;

            #[inline]
            fn from_native_isize(
                value: isize,
            ) -> Result<Self::Isize, Self::FromNativeError> {
                Ok(Self::Isize::from_ne(value.try_into()?))
            }

            #[inline]
            fn from_native_usize(
                value: usize,
            ) -> Result<Self::Usize, Self::FromNativeError> {
                Ok(Self::Usize::from_ne(value.try_into()?))
            }
        }
        le {
            type ToNativeError = ::core::convert::Infallible;

            #[inline]
            fn to_native_isize(
                value: Self::Isize,
            ) -> Result<isize, Self::ToNativeError> {
                // TODO const_num_from_num: Use `isize::TryFrom` when const.
                #[allow(clippy::as_conversions)]
                Ok(value.to_ne() as isize)
            }

            #[inline]
            fn to_native_usize(
                value: Self::Usize,
            ) -> Result<usize, Self::ToNativeError> {
                // TODO const_num_from_num: Use `usize::TryFrom` when const.
                #[allow(clippy::as_conversions)]
                Ok(value.to_ne() as usize)
            }
        }
        ge {
            type FromNativeError = ::core::convert::Infallible;

            #[inline]
            fn from_native_isize(
                value: isize,
            ) -> Result<Self::Isize, Self::FromNativeError> {
                // TODO const_num_from_num: Use `isize::TryInto` when const.
                #[allow(clippy::as_conversions)]
                Ok(Self::Isize::from_ne(value as Self::Isize))
            }

            #[inline]
            fn from_native_usize(
                value: isize,
            ) -> Result<Self::Usize, Self::FromNativeError> {
                // TODO const_num_from_num: Use `usize::TryInto` when const.
                #[allow(clippy::as_conversions)]
                Ok(Self::Usize::from_ne(value as Self::Usize))
            }
        }
        gt {
            type ToNativeError = ::core::num::TryFromIntError;

            #[inline]
            fn to_native_isize(
                value: Self::Isize,
            ) -> Result<isize, Self::ToNativeError> {
                Ok(value.to_ne().try_into()?)
            }

            #[inline]
            fn to_native_usize(
                value: Self::Usize,
            ) -> Result<usize, Self::ToNativeError> {
                Ok(value.to_ne().try_into()?)
            }
        }
    }
}
