use ::core::{alloc::Layout, ptr::NonNull};
use ::heresy::alloc::AllocError;

use crate::Ref;

/// An implementation of `RawAllocator` can allocate, grow, shrink, and
/// deallocate arbitrary blocks of data described via `Layout`.
///
/// # Safety
///
/// Memory blocks returned from an allocator must point to valid memory and
/// retain their validity until the block is deallocated, or the allocator is
/// dropped or rendered inaccessible, whichever comes first.
pub unsafe trait RawAllocator {
    /// Attempts to allocate a block of memory.
    ///
    /// On success, returns a `NonNull<[u8]>` meeting the size and alignment
    /// guarantees of `layout`.
    ///
    /// The returned block may have a larger size than specified by
    /// `layout.size()`, and may or may not have its contents initialized.
    ///
    /// # Errors
    ///
    /// Returning `Err` indicates that either memory is exhausted or `layout`
    /// does not meet an allocator's size or alignment constraints.
    ///
    /// Implementations are encouraged to return `Err` on memory exhaustion
    /// rather than panicking or aborting, but this is not a strict requirement.
    /// (Specifically: it is _legal_ to implement this trait atop an underlying
    /// native allocation library that aborts on memory exhaustion.)
    fn raw_allocate(
        this: Ref<'_, Self>,
        layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError>;

    /// Deallocates the memory referenced by `ptr`.
    ///
    /// # Safety
    ///
    /// - `ptr` must denote a block of memory _currently allocated_ via this
    ///    allocator.
    /// - `layout` must _fit_ that block of memory.
    unsafe fn raw_deallocate(
        this: Ref<'_, Self>,
        ptr: NonNull<u8>,
        layout: Layout,
    );

    /// Behaves like `raw_allocate`, but also ensures that the returned memory is
    /// zero-initialized.
    ///
    /// # Errors
    ///
    /// See [`allocate`](RawAllocator::raw_allocate) for errors.
    fn raw_allocate_zeroed(
        this: Ref<'_, Self>,
        layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        let ptr = Self::raw_allocate(this, layout)?;
        let len = ::ptr_meta::metadata(ptr.as_ptr());
        // SAFETY: `alloc` returned a valid memory block of length `len`.
        unsafe {
            ptr.as_ptr().cast::<u8>().write_bytes(0, len);
        }
        Ok(ptr)
    }

    /// Attempts to extend the memory block.
    ///
    /// Returns a new `NonNull<[u8]>` containing a pointer and the actual size
    /// of the allocated memory. The pointer is suitable for holding data
    /// described by new_layout. To accomplish this, the allocator may extend
    /// the allocation referenced by `ptr` to fit the new layout.
    ///
    /// If this returns `Ok`, then ownership of the memory block referenced by
    /// `ptr` has been transferred to this allocator. The memory may or may not
    /// have been freed, and should be considered unusable unless it was
    /// transferred back to the caller again via the return value of this
    /// method.
    ///
    /// If this method returns `Err`, then ownership of the memory block has not
    /// been transferred to this allocator, and the contents of the memory block
    /// are unaltered.
    ///
    /// # Safety
    ///
    /// - `ptr` must denote a block of memory _currently allocated_ via this
    ///   allocator.
    /// - `old_layout` must _fit_ that block of memory (the `new_layout`
    ///   argument need not fit it).
    /// - `new_layout.size()` must be greater than or equal to
    ///   `old_layout.size()`.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the new layout does not meet the allocator's size and
    /// alignment constraints, or if growing otherwise fails.
    ///
    /// Implementations are encouraged to return `Err` on memory exhaustion
    /// rather than panicking or aborting, but this is not a strict requirement.
    /// (Specifically: it is _legal_ to implement this trait atop an underlying
    /// native allocation library that aborts on memory exhaustion).
    unsafe fn raw_grow(
        this: Ref<'_, Self>,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        debug_assert!(
            new_layout.size() >= old_layout.size(),
            "`new_layout.size()` must be greater than or equal to `old_layout.size()`",
        );

        // SAFETY: `grow_in_place` has the same safety requirements as `grow`.
        let result = unsafe {
            Self::raw_grow_in_place(this, ptr, old_layout, new_layout)
        };
        result.or_else(|_| {
            let new_ptr = Self::raw_allocate(this, new_layout)?;

            // SAFETY:
            // - The caller has guaranteed that `old_layout` fits the memory
            //   pointed to by `ptr`, and so must be valid for reads of
            //   `old_layout.size()`.
            // - The caller has guaranteed that `new_layout.size()` is
            //   greater than or equal to `old_layout.size()`, so `new_ptr`
            //   must be valid for writes of `old_layout.size()`.
            // - `u8` has an alignment of 1, so both pointers must be
            //   properly aligned.
            // - The memory pointed by `new_ptr` is freshly-allocated and
            //   must not overlap with the memory pointed to by `old_ptr`.
            unsafe {
                ::core::ptr::copy_nonoverlapping(
                    ptr.as_ptr(),
                    new_ptr.as_ptr().cast::<u8>(),
                    old_layout.size(),
                );
            }
            // SAFETY:The caller has guaranteed that `ptr` denotes a block
            // of memory currently allocated via this allocator, and that
            // `old_layout` fits that block of memory.
            unsafe {
                Self::raw_deallocate(this, ptr, old_layout);
            }

            Ok(new_ptr)
        })
    }

    /// Behaves like `raw_grow`, but also ensures that the new contents are set
    /// to zero before being returned.
    ///
    /// The memory block will contain the following contents after a successful
    /// call to `grow_zeroed`:
    ///
    /// - Bytes `0..old_layout.size()` are preserved from the original
    ///   allocation.
    /// - Bytes `old_layout.size()..old_size` will either be preserved or
    ///   zeroed, depending on the allocator implementation. `old_size` refers
    ///   to the size of the memory block prior to the `grow_zeroed` call, which
    ///   may be larger than the size that was originally requested when it was
    ///   allocated.
    /// - Bytes `old_size..new_size` are zeroed. `new_size` refers to the size
    ///   of the memory block returned by the `grow_zeroed` call.
    ///
    /// # Safety
    ///
    /// - `ptr` must denote a block of memory currently allocated via this
    ///   allocator.
    /// - `old_layout` must _fit_ that block of memory (the `new_layout`
    ///   argument need not fit it).
    /// - `new_layout.size()` must be greater than or equal to
    ///   `old_layout.align()`.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the new layout does not meet the allocator's size and
    /// alignment constraints, or if growing otherwise fails.
    ///
    /// Implementations are encouraged to return `Err` on memory exhaustion
    /// rather than panicking or aborting, but this is not a strict requirement.
    /// (Specifically: it is _legal_ to implement this trait atop an underlying
    /// native allocation library that aborts on memory exhaustion).
    unsafe fn raw_grow_zeroed(
        this: Ref<'_, Self>,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        debug_assert!(
            new_layout.size() >= old_layout.size(),
            "`new_layout.size()` must be greater than or equal to `old_layout.size()`",
        );

        let result =
            // SAFETY: `grow_zeroed_in_place` has the same safety requirements
            // as `grow_zeroed`.
            unsafe { Self::raw_grow_zeroed_in_place(this, ptr, old_layout, new_layout) };
        result.or_else(|_| {
            let new_ptr = Self::raw_allocate(this, new_layout)?;

            // SAFETY:
            // - The caller has guaranteed that `old_layout` fits the memory
            //   pointed to by `ptr`, and so must be valid for reads of
            //   `old_layout.size()`.
            // - The caller has guaranteed that `new_layout.size()` is greater
            //   than or equal to `old_layout.size()`, so `new_ptr` must be
            //   valid for writes of `old_layout.size()`.
            // - `u8` has an alignment of 1, so both pointers must be properly
            //   aligned.
            // - The memory pointed by `new_ptr` is freshly-allocated and must
            //   not overlap with the memory pointed to by `old_ptr`.
            unsafe {
                ::core::ptr::copy_nonoverlapping(
                    ptr.as_ptr(),
                    new_ptr.as_ptr().cast::<u8>(),
                    old_layout.size(),
                );
            }
            // SAFETY:
            // - The end of the old bytes is followed by `new_size - old_size`
            //   bytes which are valid for writes.
            // - A `u8` pointer is always properly aligned.
            unsafe {
                ::core::ptr::write_bytes(
                    new_ptr.as_ptr().cast::<u8>().add(old_layout.size()),
                    0,
                    new_layout.size() - old_layout.size(),
                );
            }
            // SAFETY:The caller has guaranteed that `ptr` denotes a block of
            // memory currently allocated via this allocator, and that
            // `old_layout` fits that block of memory.
            unsafe {
                Self::raw_deallocate(this, ptr, old_layout);
            }

            Ok(new_ptr)
        })
    }

    /// Behaves like `grow` but returns `Err` if the memory block cannot be
    /// grown in-place.
    ///
    /// # Safety
    ///
    /// - `ptr` must denote a block of memory _currently allocated_ via this
    ///   allocator.
    /// - `old_layout` must _fit_ that block of memory (the `new_layout`
    ///   argument need not fit it).
    /// - `new_layout.size()` must be greater than or equal to
    ///   `old_layout.size()`.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the new layout does not meet the allocator's size and
    /// alignment constraints, or if growing in place otherwise fails.
    ///
    /// Implementations are encouraged to return `Err` on memory exhaustion
    /// rather than panicking or aborting, but this is not a strict requirement.
    /// (Specifically: it is _legal_ to implement this trait atop an underlying
    /// native allocation library that aborts on memory exhaustion).
    unsafe fn raw_grow_in_place(
        _: Ref<'_, Self>,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        let _ = (ptr, old_layout, new_layout);

        Err(AllocError)
    }

    /// Behaves like `raw_grow_zeroed` but returns `Err` if the memory block
    /// cannot be grown in-place.
    ///
    /// # Safety
    ///
    /// - `ptr` must denote a block of memory _currently allocated_ via this
    ///   allocator.
    /// - `old_layout` must _fit_ that block of memory (the `new_layout`
    ///   argument need not fit it).
    /// - `new_layout.size()` must be greater than or equal to
    ///   `old_layout.size()`.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the new layout does not meet the allocator's size and
    /// alignment constraints, or if growing in place otherwise fails.
    ///
    /// Implementations are encouraged to return `Err` on memory exhaustion
    /// rather than panicking or aborting, but this is not a strict requirement.
    /// (Specifically: it is _legal_ to implement this trait atop an underlying
    /// native allocation library that aborts on memory exhaustion).
    unsafe fn raw_grow_zeroed_in_place(
        this: Ref<'_, Self>,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        let new_ptr =
            // SAFETY: `grow_in_place` has the same safety requirements as
            // `grow_zeroed_in_place`.
            unsafe { Self::raw_grow_in_place(this, ptr, old_layout, new_layout)? };

        // SAFETY:
        // - The end of the old bytes is followed by `new_size - old_size` bytes
        //   which are valid for writes.
        // - A `u8` pointer is always properly aligned.
        unsafe {
            ::core::ptr::write_bytes(
                new_ptr.as_ptr().cast::<u8>().add(old_layout.size()),
                0,
                new_layout.size() - old_layout.size(),
            );
        }

        Ok(new_ptr)
    }

    /// Attempts to shrink the memory block.
    ///
    /// Returns a new [`NonNull<[u8]>`](NonNull) containing a pointer and the
    /// actual size of the allocated memory. The pointer is suitable for holding
    /// data described by `new_layout`. To accomplish this, the allocator may
    /// shrink the allocation referenced by `ptr` to fit the new layout.
    ///
    /// If this returns `Ok`, then ownership of the memory block referenced by
    /// `ptr` has been transferred to this allocator. The memory may or may not
    /// have been freed, and should be considered unusable unless it was
    /// transferred back to the caller again via the return value of this
    /// method.
    ///
    /// If this returns `Err`, then ownership of the memory block has not been
    /// transferred to this allocator, and the contents of the memory block are
    /// unaltered.
    ///
    /// # Safety
    ///
    /// - `ptr` must denote a block of memory _currently allocated_ by this
    ///   allocator.
    /// - `old_layout` must _fit_ that block of memory (The `new_layout`
    ///   argument need not fit it).
    /// - `new_layout.size()` must be less than or equal to `old_layout.size()`.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the new layout does not meet the allocator's size and
    /// alignment constraints, or if shrinking otherwise fails.
    ///
    /// Implementations are encouraged to return `Err` on memory exhaustion
    /// rather than panicking or aborting, but this is not a strict requirement.
    /// (Specifically: it is _legal_ to implement this trait atop an underlying
    /// native allocation library that aborts on memory exhaustion).
    unsafe fn raw_shrink(
        this: Ref<'_, Self>,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        debug_assert!(
            new_layout.size() <= old_layout.size(),
            "`new_layout.size()` must be less than or equal to `old_layout.size()`",
        );

        let result =
            // SAFETY: `shrink_in_place` has the same safety requirements as
            // `shrink`.
            unsafe { Self::raw_shrink_in_place(this, ptr, old_layout, new_layout) };
        result.or_else(|_| {
            let new_ptr = Self::raw_allocate(this, new_layout)?;

            // SAFETY:
            // - The caller has guaranteed that `old_layout` fits the memory
            //   pointed to by `ptr`, and `new_layout.size()` is less than or
            //   equal to `old_layout.size()`, so `ptr` must be valid for reads
            //   of `new_layout.size()`.
            // - `new_ptr` points to a memory block at least `new_layout.size()`
            //   in length, so `new_ptr` must be valid for writes of
            //   `new_layout.size()`.
            // - `u8` has an alignment of 1, so both pointers must be properly
            //   aligned.
            // - The memory pointed by `new_ptr` is freshly-allocated and must
            //   not overlap with the memory pointed to by `old_ptr`.
            unsafe {
                ::core::ptr::copy_nonoverlapping(
                    ptr.as_ptr(),
                    new_ptr.as_ptr().cast::<u8>(),
                    new_layout.size(),
                );
            }
            // SAFETY: The caller has guaranteed that `ptr` denotes a block of
            // memory currently allocated via this allocator, and that
            // `old_layout` fits that block of memory.
            unsafe {
                Self::raw_deallocate(this, ptr, old_layout);
            }

            Ok(new_ptr)
        })
    }

    /// Behaves like `raw_shrink` but returns `Err` if the memory block cannot
    /// be shrunk in-place.
    ///
    /// # Safety
    ///
    /// - `ptr` must denote a block of memory _currently allocated_ by this
    ///   allocator.
    /// - `old_layout` must _fit_ that block of memory (The `new_layout`
    ///   argument need not fit it).
    /// - `new_layout.size()` must be less than or equal to `old_layout.size()`.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the new layout does not meet the allocator's size and
    /// alignment constraints, or if shrinking otherwise fails.
    ///
    /// Implementations are encouraged to return `Err` on memory exhaustion
    /// rather than panicking or aborting, but this is not a strict requirement.
    /// (Specifically: it is _legal_ to implement this trait atop an underlying
    /// native allocation library that aborts on memory exhaustion).
    unsafe fn raw_shrink_in_place(
        _: Ref<'_, Self>,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        let _ = (ptr, old_layout, new_layout);

        Err(AllocError)
    }
}
