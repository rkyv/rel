use ::core::{alloc::Layout, hash::Hash};
use ::ptr_meta::{metadata, DynMetadata, Pointee};

/// Pointer metadata that can determine the memory layout of its pointee.
///
/// # Safety
///
/// `pointee_layout` must return the correct layout of a `T` pointee with this
/// metadata.
pub unsafe trait Metadata<T: Pointee<Metadata = Self> + ?Sized>:
    Copy + Send + Sync + Ord + Hash + Unpin
{
    /// Returns the layout of a `T` pointee with the this metadata.
    ///
    /// # Safety
    ///
    /// `self` must be valid metadata for `T`.
    unsafe fn pointee_layout(self) -> Layout;
}

// SAFETY: `Layout::new` returns the correct layout for `T`.
unsafe impl<T> Metadata<T> for () {
    unsafe fn pointee_layout(self) -> Layout {
        Layout::new::<T>()
    }
}

// SAFETY: `Layout::array` returns the correct layout for `[T]` because slices
// have the same layouts as arrays of the same length.
unsafe impl<T> Metadata<[T]> for usize {
    unsafe fn pointee_layout(self) -> Layout {
        // SAFETY: `Layout::array` cannot fail because `self` is valid metadata
        // for `[T]`.
        unsafe { Layout::array::<T>(self).unwrap_unchecked() }
    }
}

// SAFETY: `Layout::array` returns the correct layout for `str` because string
// slices have the same layouts as byte arrays of the same length.
unsafe impl Metadata<str> for usize {
    unsafe fn pointee_layout(self) -> Layout {
        // SAFETY: `Layout::array` cannot fail because `self` is valid metadata
        // for `str`.
        unsafe { Layout::array::<u8>(self).unwrap_unchecked() }
    }
}

// SAFETY: `DynMetadata::layout` returns the layout of the corresponding value.
unsafe impl<T> Metadata<T> for DynMetadata<T>
where
    T: Pointee<Metadata = DynMetadata<T>> + ?Sized,
{
    unsafe fn pointee_layout(self) -> Layout {
        self.layout()
    }
}

/// Returns the layout of the value pointed to by the given pointer.
pub fn layout_of_val_raw<T: Pointee + ?Sized>(ptr: *const T) -> Layout
where
    T::Metadata: Metadata<T>,
{
    let metadata = metadata(ptr);
    // SAFETY: `metadata` is the metadata of a pointer, and all pointers must
    // have valid metadata. So the metadata is guaranteed to be valid.
    unsafe { metadata.pointee_layout() }
}
