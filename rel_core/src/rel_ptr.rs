//! Relative pointers and related types.

use ::core::{
    marker::{PhantomData, PhantomPinned},
    mem::MaybeUninit,
};
use ::mischief::{In, Region, Slot};
use ::munge::munge;
use ::ptr_meta::{metadata, Pointee};
use ::situ::{DropRaw, Mut, Pinned, Ref, Val};

use crate::{
    rel_mem,
    Basis,
    BasisPointee,
    Emplace,
    EmplaceExt,
    Move,
    Portable,
};

/// A pointer that stores the difference between itself and its pointee.
#[repr(C)]
#[derive(DropRaw, Portable)]
#[rel_core = "crate"]
pub struct RelPtr<T: BasisPointee<B> + ?Sized, R: Region, B: Basis> {
    offset: B::Isize,
    metadata: MaybeUninit<<T as BasisPointee<B>>::BasisMetadata>,
    _phantom: PhantomData<(*mut T, R)>,
    _pinned: PhantomPinned,
}

impl<T: BasisPointee<B> + ?Sized, R: Region, B: Basis> RelPtr<T, R, B> {
    /// Returns the base pointer for the relative pointer.
    ///
    /// The base of the relative pointer is always its location in memory.
    #[inline]
    pub fn base(this: Ref<'_, Self>) -> *const u8 {
        this.as_ptr().cast()
    }

    /// Returns the mutable base pointer for the relative pointer.
    ///
    /// The base of the relative pointer is always its location in memory.
    #[inline]
    pub fn base_mut(this: Mut<'_, Self>) -> *mut u8 {
        this.as_ptr().cast()
    }

    /// Returns the offset of the relative pointer's target from its base.
    #[inline]
    pub fn offset(&self) -> isize {
        B::to_native_isize(self.offset).unwrap()
    }

    /// Returns whether the offset of the relative pointer is `0`.
    #[inline]
    pub fn is_null(&self) -> bool {
        self.offset() == 0
    }

    /// Returns the metadata of the relative pointer's pointee if it is not
    /// null.
    #[inline]
    pub fn metadata(&self) -> Option<T::Metadata> {
        if self.is_null() {
            None
        } else {
            // SAFETY: This relative pointer is not null.
            unsafe { Some(self.metadata_unchecked()) }
        }
    }

    /// Returns the metadata of the relative pointer's pointee.
    ///
    /// # Safety
    ///
    /// The relative pointer must not be null.
    #[inline]
    pub unsafe fn metadata_unchecked(&self) -> T::Metadata {
        // SAFETY: Non-null relative pointers always have valid, initialized
        // metadata.
        let metadata = unsafe { self.metadata.assume_init() };
        T::to_native_metadata(metadata).unwrap()
    }

    /// Returns the target of the relative pointer if it is not null.
    #[inline]
    pub fn as_ptr(this: Ref<'_, Self>) -> Option<*const T> {
        if this.is_null() {
            None
        } else {
            // SAFETY: We checked that the relative pointer is not null.
            unsafe { Some(Self::as_ptr_unchecked(this)) }
        }
    }

    /// Returns the target of the relative pointer.
    ///
    /// # Safety
    ///
    /// The relative pointer must not be null.
    #[inline]
    pub unsafe fn as_ptr_unchecked(this: Ref<'_, Self>) -> *const T {
        // SAFETY:
        // - Relative pointers always point to in-bounds targets.
        // - `self.offset()` always fits in an `isize`.
        // - The offset of the relative pointer never leaves the contiguous
        //   memory segment in which it is located.
        let data_address = unsafe { Self::base(this).offset(this.offset()) };
        // SAFETY: The caller has guaranteed that this relative pointer is not
        // null.
        let metadata = unsafe { this.metadata_unchecked() };

        ::ptr_meta::from_raw_parts(data_address.cast(), metadata)
    }

    /// Returns a `Ref` to the target of the relative pointer.
    ///
    /// # Safety
    ///
    /// - `this` must be non-null, properly aligned, and valid for reads.
    /// - `this` must not alias any other mutable references for `'a`.
    /// - The value pointed to by `this` must be initialized.
    pub unsafe fn as_ref<'a>(this: Ref<'_, Self>) -> Ref<'a, T> {
        // SAFETY: The caller has guaranteed that `this` is non-null, properly
        // aligned, and valid for reads. They have also guaranteed that `this`
        // does not alias any other mutable references for `'a` and that the
        // value pointed to by `this` must be initialized.
        unsafe { Ref::new_unchecked(Self::as_ptr_unchecked(this)) }
    }

    /// Returns a mutable pointer to the target of the relative pointer if it is
    /// not null.
    #[inline]
    pub fn as_mut_ptr(this: Mut<'_, Self>) -> Option<*mut T> {
        if this.is_null() {
            None
        } else {
            // SAFETY: We checked that the relative pointer is not null.
            unsafe { Some(Self::as_mut_ptr_unchecked(this)) }
        }
    }

    /// Returns a mutable pointer to the target of the relative pointer.
    ///
    /// # Safety
    ///
    /// `this` must not be null.
    #[inline]
    pub unsafe fn as_mut_ptr_unchecked(mut this: Mut<'_, Self>) -> *mut T {
        let data_address =
            // SAFETY:
            // - Relative pointers always point to in-bounds targets.
            // - `self.offset()` always fits in an `isize`.
            // - The offset of the relative pointer never leaves the contiguous
            //   memory segment in which it is located.
            unsafe { Self::base_mut(this.as_mut()).offset(this.offset()) };
        // SAFETY: The caller has guaranteed that this relative pointer is not
        // null.
        let metadata = unsafe { this.metadata_unchecked() };

        ::ptr_meta::from_raw_parts_mut(data_address.cast(), metadata)
    }

    /// Returns a `Mut` to the target of the relative pointer.
    ///
    /// # Safety
    ///
    /// - `this` must be non-null, properly aligned, and valid for reads and
    ///    writes.
    /// - `this` must not alias any other accessible references for `'a`.
    /// - The pointee of `this` must be initialized and immovable.
    pub unsafe fn as_mut<'a>(this: Mut<'_, Self>) -> Mut<'a, T> {
        // SAFETY:
        // - The caller has guaranteed that `this` is non-null, properly
        //   aligned, and valid for reads and writes.
        // - The caller has guaranteed that the pointee of `this` does not alias
        //   any other accessible references for `'a` and that the pointee of
        //   `this` is initialized.
        // - The caller has guaranteed that the pointee of `this` is initialized
        //   and immovable.
        unsafe { Mut::new_unchecked(Self::as_mut_ptr_unchecked(this)) }
    }

    /// Sets the pointee of this `RelPtr`.
    pub fn set(this: Mut<'_, Self>, ptr: In<*mut T, R>) {
        rel_mem::replace(In::new(this), ptr);
    }

    /// # Safety
    ///
    /// The memory pointed to by `ptr` and `slot` must be located in the same
    /// contiguous memory segment.
    unsafe fn emplace_new(ptr: *mut T, mut out: Slot<'_, Self>) {
        let base = out.as_ptr().cast();

        // SAFETY:
        // - The caller has guaranteed that `base` and `ptr` are in bounds of
        //   the same allocated object and derived from a pointer to the same
        //   object.
        // - The size of a `u8` is 1, so the distance between the base and
        //   target is always a multiple of it.
        let offset = unsafe { ptr.cast::<u8>().offset_from(base) };
        let offset = B::from_native_isize(offset).unwrap();

        let metadata = T::from_native_metadata(metadata(ptr)).unwrap();

        munge!(
            let RelPtr {
                offset: mut out_offset,
                metadata: mut out_metadata,
                ..
            } = out.as_mut();
        );
        out_offset.write(offset);
        out_metadata.write(MaybeUninit::new(metadata));
    }

    fn emplace_null(mut slot: Slot<'_, Self>) {
        munge!(
            let RelPtr {
                offset: mut out_offset,
                metadata: mut out_metadata,
                ..
            } = slot.as_mut();
        );
        out_offset.write(B::from_native_isize(0).unwrap());
        out_metadata.write(MaybeUninit::zeroed());
    }
}

// SAFETY: Values of type `RelPtr<T, R, B>` can only be created in `R` and may
// only be moved within `R`. Therefore, all values of the type must be located
// in `R`.
unsafe impl<T, R, B> Pinned<R> for RelPtr<T, R, B>
where
    T: BasisPointee<B> + Pointee + ?Sized,
    R: Region,
    B: Basis,
{
}

// SAFETY:
// - `RelPtr` is `Sized` and always has metadata `()`, so `emplaced_meta` always
//   returns valid metadata for it.
// - `emplace_unsized_unchecked` initializes its `out` parameter.
unsafe impl<T, R, B> Emplace<RelPtr<T, R, B>, R> for In<*mut T, R>
where
    T: BasisPointee<B> + Pointee + ?Sized,
    R: Region,
    B: Basis,
{
    fn emplaced_meta(&self) -> <RelPtr<T, R, B> as Pointee>::Metadata {}

    unsafe fn emplace_unsized_unchecked(
        self,
        out: In<Slot<'_, RelPtr<T, R, B>>, R>,
    ) {
        let ptr = In::into_inner(self);

        // SAFETY: `emplace_new` returns the same slot, but initialized.
        // Therefore it must be located in the same region.
        unsafe {
            RelPtr::emplace_new(ptr, In::into_inner(out));
        }
    }
}

/// An emplacer for a null `RelPtr`.
pub struct Null;

// SAFETY:
// - `RelPtr` is `Sized` and always has metadata `()`, so `emplaced_meta` always
//   returns valid metadata for it.
// - `emplace_unsized_unchecked` initializes its `out` parameter.
unsafe impl<T, R, B> Emplace<RelPtr<T, R, B>, R> for Null
where
    T: BasisPointee<B> + ?Sized,
    R: Region,
    B: Basis,
{
    fn emplaced_meta(&self) -> <RelPtr<T, R, B> as Pointee>::Metadata {}

    unsafe fn emplace_unsized_unchecked(
        self,
        out: In<Slot<'_, RelPtr<T, R, B>>, R>,
    ) {
        RelPtr::emplace_null(In::into_inner(out));
    }
}

// SAFETY: `move_unsized_unchecked` initializes `out` by emplacing into it.
unsafe impl<T, R, B> Move<R> for RelPtr<T, R, B>
where
    T: BasisPointee<B> + ?Sized,
    R: Region,
    B: Basis,
{
    unsafe fn move_unsized_unchecked(
        this: In<Val<'_, Self>, R>,
        out: In<Slot<'_, Self>, R>,
    ) {
        let mut this = In::into_inner(this);
        if let Some(ptr) = RelPtr::as_mut_ptr(this.as_mut()) {
            drop(this);
            // SAFETY: `ptr` was unwrapped from an `In` and was not mutated, so
            // its pointee is still located in `R`.
            let ptr = unsafe { In::new_unchecked(ptr) };
            ptr.emplace(out);
        } else {
            Null.emplace(out);
        }
    }
}
