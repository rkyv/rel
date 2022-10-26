use ::core::{
    mem::{ManuallyDrop, MaybeUninit},
    ops::{Deref, DerefMut},
    ptr::{self, read, NonNull},
};
#[cfg(feature = "alloc")]
use ::heresy::alloc::Global;
use ::heresy::{alloc::Allocator, Box};
use ::ptr_meta::Pointee;

use crate::{
    layout_of_val_raw,
    Metadata,
    Pointer,
    RegionalAllocator,
    Slot,
    Within,
};

/// A box that may be uninitialized.
///
/// Frames have the same memory layouts as [`Boxes`](Box).
pub struct Frame<
    T: Pointee + ?Sized,
    #[cfg(feature = "alloc")] A: Allocator = ::heresy::alloc::Global,
    #[cfg(not(feature = "alloc"))] A: Allocator,
> where
    T::Metadata: Metadata<T>,
{
    ptr: NonNull<T>,
    alloc: A,
}

impl<T: Pointee + ?Sized, A: Allocator> Drop for Frame<T, A>
where
    T::Metadata: Metadata<T>,
{
    fn drop(&mut self) {
        let layout = layout_of_val_raw(self.as_ptr());
        // SAFETY:
        // - `ptr` is currently allocated by `self.alloc`.
        // - `layout` fits that block of memory.
        unsafe {
            self.alloc.deallocate(self.ptr.cast(), layout);
        }
    }
}

impl<T: Pointee + ?Sized, A: Allocator> Frame<T, A>
where
    T::Metadata: Metadata<T>,
{
    /// Returns a reference to the underlying allocator.
    ///
    /// Note: this is an associated function, which means that you have to call
    /// it as `Frame::allocator(&f)` instead of `f.allocator()`. This is so that
    /// there is no conflict with a method on the inner type.
    pub fn allocator(f: &Self) -> &A {
        &f.alloc
    }

    /// Returns a pointer to the underlying memory.
    pub fn as_ptr(&self) -> *const T {
        self.ptr.as_ptr()
    }

    /// Returns a mutable pointer to the underlying memory.
    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.ptr.as_ptr()
    }

    /// Converts to `Box<T, A>`.
    ///
    /// This does not move the value pointed to by the frame.
    ///
    /// # Safety
    ///
    /// The contents of the frame must be initialized. Calling this when the
    /// content is not yet initialized causes immediate undefined behavior.
    ///
    /// Additionally, most types have additional invariants beyond merely being
    /// considered initialized at the type level. For example, a `1`-initialized
    /// `Vec<T>` is considered initialized (under the current implementation;
    /// this does not constitute a stable guarantee) because the only
    /// requirement the compiler knows about it is that the data pointer must be
    /// non-null. Creating such a `Vec<T>` does not cause _immediate_ undefined
    /// behavior, but will cause undefined behavior with most safe operations
    /// (including dropping it).
    pub unsafe fn assume_init(self) -> Box<T, A> {
        let (raw, alloc) = Self::into_raw_with_allocator(self);
        // SAFETY: `Frame`s allocate memory with the same layouts as `Box`es.
        unsafe { Box::from_raw_in(raw, alloc) }
    }

    /// Constructs a frame from a raw pointer in the given allocator.
    ///
    /// After calling this function, the raw pointer is owned by the resulting
    /// `Frame`. Specifically, the `Frame` destructor will free the allocated
    /// memory. For this to be safe, the memory must have been allocated in
    /// accordance with the memory layout used by `Frame`.
    ///
    /// # Safety
    ///
    /// `raw` must be non-null and allocated by `alloc` according to the memory
    /// layout used by `Frame`.
    pub unsafe fn from_raw_in(raw: *mut T, alloc: A) -> Self {
        Self {
            // SAFETY: `raw` is not null.
            ptr: unsafe { NonNull::new_unchecked(raw) },
            alloc,
        }
    }

    /// Consumes the `Frame`, returning a raw pointer and the allocator.
    ///
    /// The pointer will be properly aligned and non-null.
    ///
    /// After calling this function, the caller is responsible for the memory
    /// previously managed by the `Frame`. In particular, the caller should
    /// properly release the memory, taking into account the memory layout used
    /// by `Frame`. The easiest way to do this is to convert the raw pointer
    /// back into a `Frame` with the [`Frame::from_raw_in`] function, allowing
    /// the `Frame` destructor to perform the cleanup.
    ///
    /// Note: this is an associated function, which means that you have to call
    /// it as `Frame::into_raw_with_allocator(f)` instead of
    /// `f.into_raw_with_allocator()`. This is so that there is no conflict with
    /// a method on the inner type.
    pub fn into_raw_with_allocator(f: Self) -> (*mut T, A) {
        let f = ManuallyDrop::new(f);
        // SAFETY: Because it is a reference, `f.alloc` is valid for reads,
        // properly aligned, and points to a properly initialized `A`. The
        // original will never be accessed or dropped.
        (f.ptr.as_ptr(), unsafe { read(&f.alloc) })
    }

    /// Allocates memory for an unsized type with the given metadata in the
    /// given allocator.
    ///
    /// This doesn't actually allocate if the metadata provides a layout with
    /// zero size.
    ///
    /// # Safety
    ///
    /// `metadata` must be valid for a pointer to `T`.
    pub unsafe fn new_unsized_in(metadata: T::Metadata, alloc: A) -> Self {
        // SAFETY: The caller has ensured that `metadata` is valid for a pointer
        // to `T`.
        let layout = unsafe { metadata.pointee_layout() };
        let ptr = if layout.size() == 0 {
            // TODO strict_provenance: Use `::core::ptr::invalid_mut`.
            #[allow(clippy::as_conversions)]
            ptr_meta::from_raw_parts_mut(layout.align() as *mut (), metadata)
        } else {
            ptr_meta::from_raw_parts_mut(
                alloc.allocate(layout).unwrap().as_ptr().cast(),
                metadata,
            )
        };
        Self {
            // SAFETY: `ptr` is non-null.
            ptr: unsafe { NonNull::new_unchecked(ptr) },
            alloc,
        }
    }

    /// Returns a [`Slot`] of the internal contents.
    pub fn slot(&'_ mut self) -> Slot<'_, T> {
        // SAFETY:
        // - `self.as_mut_ptr()` is always non-null, properly aligned, and valid
        //   for reads and writes.
        // - `self.as_mut_ptr()` cannot alias any other accessible references
        //   because `self` is mutably borrowed.
        unsafe { Slot::new_unchecked(self.as_mut_ptr()) }
    }
}

impl<T, A: Allocator> Frame<T, A> {
    /// Converts a `Frame<T>` into a `Frame<[T]>`.
    ///
    /// This conversion does not allocate on the heap and happens in place.
    pub fn into_framed_slice(framed: Self) -> Frame<[T], A> {
        let (ptr, alloc) = Self::into_raw_with_allocator(framed);
        let ptr = ptr::slice_from_raw_parts_mut(ptr.cast(), 1);

        // SAFETY: `ptr` is a valid slice of length 1 allocated in `alloc`.
        unsafe { Frame::from_raw_in(ptr, alloc) }
    }

    /// Allocates memory in the given allocator.
    ///
    /// This doesn't actually allocate if `T` is zero-sized.
    pub fn new_in(alloc: A) -> Self {
        // SAFETY: `()` is valid metadata for a pointer to `T`.
        unsafe { Self::new_unsized_in((), alloc) }
    }

    /// Sets the value of the underlying memory and converts the frame to a box.
    ///
    /// This overwrites any previous value without dropping it, so be careful
    /// not to use this after initializing the frame unless you want to skip
    /// running the destructor.
    pub fn init(mut self, value: T) -> Box<T, A> {
        self.write(value);
        // SAFETY: The value in this frame is now initialized.
        unsafe { self.assume_init() }
    }
}

#[cfg(feature = "alloc")]
impl<T: Pointee + ?Sized> Frame<T, Global>
where
    T::Metadata: Metadata<T>,
{
    /// Constructs a frame from a raw pointer.
    ///
    /// After calling this function, the raw pointer is owned by the resulting
    /// `Frame`. Specifically, the `Frame` destructor will free the allocated
    /// memory. For this to be safe, the memory must have been allocated in
    /// accordance with the memory layout used by `Frame`.
    ///
    /// # Safety
    ///
    /// This function is unsafe because improper use may lead to memory
    /// problems. For example, a double-free may occur if the function is called
    /// twice on the same raw pointer.
    ///
    /// The safety conditions are described in the memory layout section.
    pub unsafe fn from_raw(raw: *mut T) -> Self {
        // SAFETY: The safety requirements for this function are the same as
        // those for `from_raw_in`.
        unsafe { Self::from_raw_in(raw, Global) }
    }

    /// Consumes the `Frame`, returning a raw pointer.
    ///
    /// The pointer will be properly aligned and non-null.
    ///
    /// After calling this function, the caller is responsible for the memory
    /// previously managed by the `Frame`. In particular, the caller should
    /// properly release the memory, taking into account the memory layout used
    /// by `Frame`. The easiest way to do this is to convert the raw pointer
    /// back into a `Frame` with the [`Frame::from_raw`] function, allowing the
    /// `Frame` destructor to perform the cleanup.
    ///
    /// Note: this is an associated function, which means that you have to call
    /// it as `Frame::into_raw(f)` instead of `f.into_raw()`. This is so that
    /// there is no conflict with a method on the inner type.
    pub fn into_raw(f: Self) -> *mut T {
        Frame::into_raw_with_allocator(f).0
    }

    /// Allocates memory for an object with the given metadata on the heap.
    ///
    /// This doesn't actually allocate if the metadata provides a layout with
    /// zero size.
    ///
    /// # Safety
    ///
    /// `metadata` must be valid for a pointer to `T`.
    pub unsafe fn new_unsized(metadata: T::Metadata) -> Self {
        // SAFETY: The caller has guaranteed that `metadata` is valid for a
        // pointer to `T`.
        unsafe { Self::new_unsized_in(metadata, Global) }
    }
}

#[cfg(feature = "alloc")]
impl<T> Frame<T, Global> {
    /// Allocates memory on the heap.
    ///
    /// This doesn't actually allocate if `T` is zero-sized.
    pub fn new() -> Self {
        Self::new_in(Global)
    }
}

impl<T: Default, A: Allocator + Default> Default for Frame<T, A> {
    fn default() -> Self {
        Self::new_in(A::default())
    }
}

// SAFETY: `Frame` is guaranteed to return the same value from `target`,
// `deref`, and `deref_mut`.
unsafe impl<T: Pointee + ?Sized, A: Allocator> Pointer for Frame<T, A>
where
    <T as Pointee>::Metadata: Metadata<T>,
{
    type Target = T;

    fn target(&self) -> *mut Self::Target {
        self.as_ptr().cast_mut()
    }
}

impl<T, A: Allocator> Deref for Frame<T, A> {
    type Target = MaybeUninit<T>;

    fn deref(&self) -> &Self::Target {
        // SAFETY: `self.as_ptr()` is non-null, properly aligned, and always a
        // valid `MaybeUninit<T>`.
        unsafe { &*self.as_ptr().cast() }
    }
}

impl<T, A: Allocator> DerefMut for Frame<T, A> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY:
        // - `self.as_mut_ptr()` is non-null, properly aligned, and always a
        //   valid `MaybeUninit<T>`.
        // - `self.as_mut_ptr()` cannot be aliased because `self` is mutably
        //   borrowed.
        unsafe { &mut *self.as_mut_ptr().cast() }
    }
}

// SAFETY: The pointee of `Frame<T, A>` is located in `A::Region` because it
// is allocated in `A`.
unsafe impl<T, A> Within<A::Region> for Frame<T, A>
where
    T: Pointee + ?Sized,
    <T as Pointee>::Metadata: Metadata<T>,
    A: RegionalAllocator,
{
}

#[cfg(all(test, feature = "alloc"))]
mod tests {
    use ::munge::munge;

    use crate::Frame;

    #[test]
    fn munge() {
        use crate::Frame;

        struct Example {
            a: i32,
            b: char,
        }

        let mut frame = Frame::<Example>::new();
        let mut slot = frame.slot();

        munge!(let Example { mut a, mut b } = slot.as_mut());
        assert_eq!(*a.write(42), 42);
        assert_eq!(*b.write('x'), 'x');

        // SAFETY: The slot has been initialized.
        let example = unsafe { slot.assume_init_ref() };
        assert_eq!(example.a, 42);
        assert_eq!(example.b, 'x');

        // SAFETY: The value in frame is initialized.
        let value = unsafe { frame.assume_init() };
        assert_eq!(value.a, 42);
        assert_eq!(value.b, 'x');
    }

    #[test]
    fn basic_functionality() {
        let mut x = Frame::<char>::new();
        assert_eq!(*x.write('a'), 'a');
        // SAFETY: `x` is initialized.
        let x = unsafe { x.assume_init() };
        assert_eq!(*x, 'a');
    }

    #[test]
    fn str_frame() {
        // SAFETY: `5` is valid metadata for a `str`.
        let mut x = unsafe { Frame::<str>::new_unsized(5) };
        assert_eq!(::ptr_meta::metadata(x.as_mut_ptr()), 5);
        // SAFETY: `x` is a valid `[u8; 5]`.
        unsafe {
            x.as_mut_ptr().cast::<[u8; 5]>().write(*b"hello");
        }
        // SAFETY: `x` is initialized.
        let s = unsafe { x.assume_init() };
        assert_eq!(&*s, "hello");
    }

    #[test]
    fn slice_frame() {
        // SAFETY: `4` is valid metadata for a `[u32]`.
        let mut x = unsafe { Frame::<[u32]>::new_unsized(4) };
        assert_eq!(::ptr_meta::metadata(x.as_mut_ptr()), 4);
        // SAFETY: `x` is a valid `[u32; 4]`.
        unsafe {
            x.as_mut_ptr().cast::<[u32; 4]>().write([1, 2, 3, 4]);
        }
        // SAFETY: `x` is initialized.
        let s = unsafe { x.assume_init() };
        assert_eq!(&*s, [1, 2, 3, 4]);
    }

    #[test]
    fn zeroed_slice() {
        // SAFETY: `4` is valid metadata for a `[u32]`.
        let mut x = unsafe { Frame::<[u32]>::new_unsized(4) };
        x.slot().zero();
        // SAFETY: `x` has been completely zeroed, which has initialized all 4
        // elements to `0`.
        let s = unsafe { x.assume_init() };
        assert_eq!(&*s, [0, 0, 0, 0]);
    }
}
