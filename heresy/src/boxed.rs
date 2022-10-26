//! A pointer type for memory allocation.

use ::core::{
    alloc::Layout,
    fmt::{Debug, Display},
    mem::ManuallyDrop,
    ops::{Deref, DerefMut},
    ptr::{read, NonNull},
};

use crate::alloc::Allocator;
#[cfg(feature = "alloc")]
use crate::alloc::Global;

/// A pointer type for memory allocation.
pub struct Box<
    T: ?Sized,
    #[cfg(feature = "alloc")] A: Allocator = Global,
    #[cfg(not(feature = "alloc"))] A: Allocator,
> {
    ptr: NonNull<T>,
    alloc: A,
}

impl<T: ?Sized, A: Allocator> Drop for Box<T, A> {
    fn drop(&mut self) {
        let layout = Layout::for_value(&**self);
        // SAFETY: `self.ptr` is always valid for reads and writes, properly
        // aligned, and valid for dropping.
        unsafe {
            self.ptr.as_ptr().drop_in_place();
        }
        // SAFETY: `self.ptr` is always allocated via `self.alloc` and `layout`
        // is guaranteed to fit it.
        unsafe {
            self.alloc.deallocate(self.ptr.cast(), layout);
        }
    }
}

impl<T: ?Sized, A: Allocator> Box<T, A> {
    /// Constructs a box from a raw pointer in the given allocator.
    ///
    /// After calling this function, the raw pointer is owned by the resulting
    /// `Box`. Specifically, the `Box` destructor will call the destructor of
    /// `T` and free the allocated memory. For this to be safe, the memory must
    /// have been allocated in accordance with the memory layout used by `Box`.
    ///
    /// # Safety
    ///
    /// - `ptr` must point to a memory block currently allocated by `alloc`.
    /// - The layout used to allocate `ptr` must exactly match the return value
    ///   of `Layout::for_value`.
    /// - `ptr` must point to an initialized `T`.
    pub unsafe fn from_raw_in(ptr: *mut T, alloc: A) -> Self {
        Self {
            // SAFETY: `ptr` is allocated by `alloc`, and so must not be null.
            ptr: unsafe { NonNull::new_unchecked(ptr) },
            alloc,
        }
    }

    /// Consumes the `Box`, returning a wrapped raw pointer and the allocator.
    ///
    /// The pointer will be properly aligned and non-null.
    ///
    /// After calling this function, the caller is responsible for the memory
    /// previously managed by the `Box`. In particular, the caller should
    /// properly destroy `T` and release the memory, taking into account the
    /// memory layout used by `Box`. The easiest way to do this is to convert
    /// the raw pointer back into a `Box` with the `Box::from_raw_in` function,
    /// allowing the `Box` destructor to perform the cleanup.
    ///
    /// Note: this is an associated function, which means that you have to call
    /// it as `Box::into_raw_with_allocator(b)` instead of
    /// `b.into_raw_with_allocator()`. This is so that there is no conflict with
    /// a method on the inner type.
    pub fn into_raw_with_allocator(b: Self) -> (*mut T, A) {
        let b = ManuallyDrop::new(b);
        // SAFETY: Because it is a reference, `b.alloc` is valid for reads,
        // properly aligned, and points to a properly initialized `A`. The
        // original will never be accessed or dropped.
        (b.ptr.as_ptr(), unsafe { read(&b.alloc) })
    }
}

impl<T, A: Allocator> Box<T, A> {
    /// Allocates memory in the given allocator then places `x` into it.
    pub fn new_in(x: T, alloc: A) -> Box<T, A> {
        let ptr = alloc
            .allocate(Layout::new::<T>())
            .unwrap()
            .as_ptr()
            .cast::<T>();
        // SAFETY: `ptr` is guaranteed to be properly aligned for a `T` and
        // valid for writes because we just allocated it.
        unsafe {
            ptr.write(x);
        }
        // SAFETY: We allocated `ptr` in `alloc` with the layout of `T` and
        // initialized it by writing `x`.
        unsafe { Self::from_raw_in(ptr, alloc) }
    }
}

#[cfg(feature = "alloc")]
impl<T> Box<T, Global> {
    /// Allocates memory in the `Global` allocator and then places `x` into it.
    pub fn new(x: T) -> Self {
        Self::new_in(x, Global)
    }
}

impl<T: Debug + ?Sized, A: Allocator> Debug for Box<T, A> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        T::fmt(self, f)
    }
}

impl<T: Display + ?Sized, A: Allocator> Display for Box<T, A> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        T::fmt(self, f)
    }
}

impl<T: ?Sized, A: Allocator> Deref for Box<T, A> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        // SAFETY:
        // - `self.ptr` is always valid for reads, properly aligned, and always
        //   points to an initialized `T`.
        // - The underlying `T` has the same aliasing as `self`.
        unsafe { &*self.ptr.as_ptr() }
    }
}

impl<T: ?Sized, A: Allocator> DerefMut for Box<T, A> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY:
        // - `self.ptr` is always valid for reads, properly aligned, and always
        //   points to an intialized `T`.
        // - The underlying `T` has the same aliasing as `self`.
        unsafe { &mut *self.ptr.as_ptr() }
    }
}
