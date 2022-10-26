//! Memory alocation APIs.

use ::core::{alloc::Layout, ptr::NonNull};

/// The `AllocError` error indicates an allocation failure that may be due to
/// resource exhaustion or to something wrong when combining the given input
/// arguments with this allocator.
#[derive(Debug)]
pub struct AllocError;

/// An implementation of `Allocator` can allocate, grow, shrink, and deallocate
/// arbitrary blocks of data described via `Layout`.
///
/// # Safety
///
/// Memory blocks returned from an allocator must point to valid memory and
/// retain their validity until the block is deallocated, or the allocator is
/// dropped or rendered inaccessible, whichever comes first.
pub unsafe trait Allocator {
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
    ///
    /// Clients wishing to abort computation in response to an allocation error
    /// are encouraged to call the [`handle_alloc_error`] function, rather than
    /// directly invoking `panic!` or similar.
    ///
    /// [`handle_alloc_error`]: ::builtin_alloc::alloc::handle_alloc_error
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError>;

    /// Deallocates the memory referenced by `ptr`.
    ///
    /// # Safety
    ///
    /// - `ptr` must denote a block of memory _currently allocated_ via this
    ///    allocator.
    /// - `layout` must _fit_ that block of memory.
    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout);

    /// Behaves like `allocate`, but also ensures that the returned memory is
    /// zero-initialized.
    ///
    /// # Errors
    ///
    /// See [`allocate`](Allocator::allocate) for errors.
    fn allocate_zeroed(
        &self,
        layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        let ptr = self.allocate(layout)?;
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
    ///
    /// Clients wishing to abort computation in response to an allocation error
    /// are encouraged to call the [`handle_alloc_error`] function, rather than
    /// directly invoking `panic!` or similar.
    ///
    /// [`handle_alloc_error`]: ::builtin_alloc::alloc::handle_alloc_error
    unsafe fn grow(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        debug_assert!(
            new_layout.size() >= old_layout.size(),
            "`new_layout.size()` must be greater than or equal to `old_layout.size()`",
        );

        // SAFETY: `grow_in_place` has the same safety requirements as `grow`.
        let result = unsafe { self.grow_in_place(ptr, old_layout, new_layout) };
        result.or_else(|_| {
            let new_ptr = self.allocate(new_layout)?;

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
                self.deallocate(ptr, old_layout);
            }

            Ok(new_ptr)
        })
    }

    /// Behaves like `grow`, but also ensures that the new contents are set to
    /// zero before being returned.
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
    ///
    /// Clients wishing to abort computation in response to an allocation error
    /// are encouraged to call the [`handle_alloc_error`] function, rather than
    /// directly invoking `panic!` or similar.
    ///
    /// [`handle_alloc_error`]: ::builtin_alloc::alloc::handle_alloc_error
    unsafe fn grow_zeroed(
        &self,
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
            unsafe { self.grow_zeroed_in_place(ptr, old_layout, new_layout) };
        result.or_else(|_| {
            let new_ptr = self.allocate(new_layout)?;

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
                self.deallocate(ptr, old_layout);
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
    ///
    /// Clients wishing to abort computation in response to an allocation error
    /// are encouraged to call the [`handle_alloc_error`] function, rather than
    /// directly invoking `panic!` or similar.
    ///
    /// [`handle_alloc_error`]: ::builtin_alloc::alloc::handle_alloc_error
    unsafe fn grow_in_place(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        let _ = (ptr, old_layout, new_layout);

        Err(AllocError)
    }

    /// Behaves like `grow_zeroed` but returns `Err` if the memory block cannot
    /// be grown in-place.
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
    ///
    /// Clients wishing to abort computation in response to an allocation error
    /// are encouraged to call the [`handle_alloc_error`] function, rather than
    /// directly invoking `panic!` or similar.
    ///
    /// [`handle_alloc_error`]: ::builtin_alloc::alloc::handle_alloc_error
    unsafe fn grow_zeroed_in_place(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        let new_ptr =
            // SAFETY: `grow_in_place` has the same safety requirements as
            // `grow_zeroed_in_place`.
            unsafe { self.grow_in_place(ptr, old_layout, new_layout)? };

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
    ///
    /// Clients wishing to abort computation in response to an allocation error
    /// are encouraged to call the [`handle_alloc_error`] function, rather than
    /// directly invoking `panic!` or similar.
    ///
    /// [`handle_alloc_error`]: ::builtin_alloc::alloc::handle_alloc_error
    unsafe fn shrink(
        &self,
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
            unsafe { self.shrink_in_place(ptr, old_layout, new_layout) };
        result.or_else(|_| {
            let new_ptr = self.allocate(new_layout)?;

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
                self.deallocate(ptr, old_layout);
            }

            Ok(new_ptr)
        })
    }

    /// Behaves like `shrink` but returns `Err` if the memory block cannot be
    /// shrunk in-place.
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
    ///
    /// Clients wishing to abort computation in response to an allocation error
    /// are encouraged to call the [`handle_alloc_error`] function, rather than
    /// directly invoking `panic!` or similar.
    ///
    /// [`handle_alloc_error`]: ::builtin_alloc::alloc::handle_alloc_error
    unsafe fn shrink_in_place(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        let _ = (ptr, old_layout, new_layout);

        Err(AllocError)
    }
}

/// The global memory allocator.
#[cfg(feature = "alloc")]
#[derive(Clone)]
pub struct Global;

#[cfg(feature = "alloc")]
impl Global {
    #[inline]
    fn dangling(align: usize) -> NonNull<[u8]> {
        // TODO strict_provenance: Use `::core::ptr::invalid_mut`.
        #[allow(clippy::as_conversions)]
        let dangling =
            // SAFETY: `layout.align()` cannot be zero.
            unsafe { NonNull::<u8>::new_unchecked(align as *mut u8) };
        let ptr = ::core::ptr::slice_from_raw_parts_mut(dangling.as_ptr(), 0);
        // SAFETY: `ptr` is non-null because the `dangling` used to create it
        // was non-null.
        unsafe { NonNull::new_unchecked(ptr) }
    }

    #[inline]
    fn alloc_impl<const ZEROED: bool>(
        &self,
        layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        match layout.size() {
            0 => Ok(Self::dangling(layout.align())),
            size => {
                let ptr = if ZEROED {
                    // SAFETY: We have checked that `layout.size()` is not zero.
                    unsafe { ::builtin_alloc::alloc::alloc_zeroed(layout) }
                } else {
                    // SAFETY: We have checked that `layout.size()` is not zero.
                    unsafe { ::builtin_alloc::alloc::alloc(layout) }
                };
                NonNull::new(::core::ptr::slice_from_raw_parts_mut(ptr, size))
                    .ok_or(AllocError)
            }
        }
    }

    #[inline]
    unsafe fn grow_impl<const ZEROED: bool>(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        use ::builtin_alloc::alloc::realloc;

        // TODO integer_constants: Use a usize constant for `isize::MAX`.
        #[allow(clippy::as_conversions)]
        const ISIZE_MAX_AS_USIZE: usize = isize::MAX as usize;

        debug_assert!(
            new_layout.size() >= old_layout.size(),
            "`new_layout.size()` must be greater than or equal to `old_layout.size()`"
        );

        let new_align = new_layout.align();

        match old_layout.size() {
            0 => self.alloc_impl::<ZEROED>(new_layout),
            _ if new_layout.size() > ISIZE_MAX_AS_USIZE - (new_align - 1) => {
                Err(AllocError)
            }
            old_size if old_layout.align() == new_align => {
                let new_size = new_layout.size();

                let ptr =
                    // SAFETY:
                    // - The caller has guaranteed that `ptr` is currently
                    //   allocated and `layout` is the same one used to allocate
                    //   `ptr`.
                    // - The caller has guaranteed that `new_layout.size() >=
                    //   old_layout.size()`, and the old size is greater than
                    //   zero, so the new size is also greater than zero.
                    // - We have checked that `new_layout.size()` does not
                    //   overflow an `isize` when rounded up to the nearest
                    //   multiple of `new_align`.
                    unsafe { realloc(ptr.as_ptr(), old_layout, new_size) };
                if !ptr.is_null() {
                    if ZEROED {
                        // SAFETY:
                        // - The caller has guaranteed that `old_size` is less
                        //   than or equal to the size of the memory block
                        //   pointed to by `ptr`.
                        // - We have asserted that `new_size` is less than
                        //   `isize::MAX` and so `old_size` must also be less
                        //   than `isize::MAX`.
                        // - `ptr + old_size` is valid for writes of `new_size -
                        //   old_size` because `realloc` extended it to at least
                        //   `new_size` in length.
                        unsafe {
                            ptr.add(old_size)
                                .write_bytes(0, new_size - old_size);
                        }
                    }
                    // SAFETY: We have checked that `ptr` is not null.
                    unsafe {
                        Ok(NonNull::new_unchecked(
                            ::core::ptr::slice_from_raw_parts_mut(
                                ptr, new_size,
                            ),
                        ))
                    }
                } else {
                    Err(AllocError)
                }
            }
            old_size => {
                let new_ptr = self.alloc_impl::<ZEROED>(new_layout)?;
                // SAFETY:
                // - The caller has guaranteed that `ptr` points to a memory
                //   block that is at least `old_size` bytes in size.
                // - `new_ptr` points to `new_layout.size()` bytes that are
                //   valid for writes, which is at least `old_size` bytes.
                // - Pointers are always aligned for reads and writes of `u8`.
                // - `ptr` and `new_ptr` are separate nonoverlapping memory
                //   blocks per the guarantees of `alloc`.
                unsafe {
                    ::core::ptr::copy_nonoverlapping(
                        ptr.as_ptr(),
                        new_ptr.as_ptr().cast(),
                        old_size,
                    );
                }
                // SAFETY: The caller has guaranteed that `ptr` points to a
                // valid memory block and that `old_layout` fits it.
                unsafe {
                    self.deallocate(ptr, old_layout);
                }
                Ok(new_ptr)
            }
        }
    }
}

#[cfg(feature = "alloc")]
// SAFETY: Memory blocks returned from `Global` point to valid memory and retain
// their validity until the block is deallocated.
unsafe impl Allocator for Global {
    #[inline]
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        self.alloc_impl::<false>(layout)
    }

    #[inline]
    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        if layout.size() != 0 {
            // SAFETY: The caller has guaranteed that `ptr` denotes a block of
            // memory currently allocated via this allocator.
            //
            // Although `layout` _fits_ the given memory block denoted by `ptr`,
            // it is not guaranteed to be the same layout used to allocate that
            // block of memory. In particular, it may use the size of the block
            // returned by `alloc` which may be larger.
            unsafe { ::builtin_alloc::alloc::dealloc(ptr.as_ptr(), layout) }
        }
    }

    #[inline]
    fn allocate_zeroed(
        &self,
        layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        self.alloc_impl::<true>(layout)
    }

    #[inline]
    unsafe fn grow(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        // SAFETY: `grow_impl` has the same safety requirements as `grow`.
        unsafe { self.grow_impl::<false>(ptr, old_layout, new_layout) }
    }

    #[inline]
    unsafe fn grow_zeroed(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        // SAFETY: `grow_impl` has the same safety requirements as `grow`.
        unsafe { self.grow_impl::<true>(ptr, old_layout, new_layout) }
    }

    #[inline]
    unsafe fn shrink(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        use ::builtin_alloc::alloc::realloc;

        debug_assert!(
            new_layout.size() <= old_layout.size(),
            "`new_layout.size()` must be smaller than or equal to `old_layout.size()`",
        );

        match new_layout.size() {
            0 => {
                // SAFETY: The caller has guaranteed that `ptr` points to a
                // valid block of memory and that `old_layout` fits it.
                unsafe {
                    self.deallocate(ptr, old_layout);
                }
                Ok(Self::dangling(new_layout.align()))
            }

            new_size if old_layout.align() == new_layout.align() => {
                let ptr =
                    // SAFETY:
                    // - The caller has guaranteed that `ptr` is currently
                    //   allocated and `old_layout` is the same one used to
                    //   allocate `ptr`.
                    // - We have checked that `new_size` is greater than zero.
                    // - `old_layout.size()` must not have overflowed an `isize`
                    //   when rounded up to the nearest multiple of
                    //   `old_layout.align()`, and `new_size` is less than
                    //   `old_layout.size()`, so `new_layout.size()` must not
                    //   overflow an `isize` when rounded up to the nearest
                    //   multiple of `old_layout.align()`.
                    unsafe { realloc(ptr.as_ptr(), old_layout, new_size) };
                NonNull::new(::core::ptr::slice_from_raw_parts_mut(
                    ptr, new_size,
                ))
                .ok_or(AllocError)
            }

            new_size => {
                let new_ptr = self.allocate(new_layout)?;
                // SAFETY:
                // - The caller has guaranteed that `ptr` points to a memory
                //   block that is at least `old_layout.size()` bytes in size,
                //   which is at least `new_size` bytes.
                // - `new_ptr` points to `new_size` bytes that are valid for
                //   writes.
                // - Pointers are always aligned for reads and writes of `u8`.
                // - `ptr` and `new_ptr` are separate nonoverlapping memory
                //   blocks per the guarantees of `allocate`.
                unsafe {
                    ::core::ptr::copy_nonoverlapping(
                        ptr.as_ptr(),
                        new_ptr.as_ptr().cast(),
                        new_size,
                    );
                }
                // SAFETY: The caller has guaranteed that `ptr` points to a
                // valid memory block and that `old_layout` fits it.
                unsafe {
                    self.deallocate(ptr, old_layout);
                }
                Ok(new_ptr)
            }
        }
    }
}
