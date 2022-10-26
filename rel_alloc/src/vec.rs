//! A contiguous growable array type with heap-allocated contents, written
//! `RelVec<T>`.

use ::core::{alloc::Layout, fmt, ptr};
use ::mischief::{In, Slot};
use ::munge::munge;
use ::ptr_meta::Pointee;
use ::rel_core::{
    Basis,
    DefaultBasis,
    Emplace,
    EmplaceExt,
    Move,
    MoveExt,
    Portable,
    RelPtr,
};
use ::situ::{
    alloc::{RawAllocator, RawRegionalAllocator},
    fmt::DebugRaw,
    ops::{DerefMutRaw, DerefRaw, IndexMutRaw, IndexRaw},
    DropRaw,
    Mut,
    Ref,
    Val,
};

use crate::alloc::RelAllocator;

/// A relative counterpart to `Vec`.
#[derive(Move, Portable)]
#[repr(C)]
pub struct RelVec<T, A: RawRegionalAllocator, B: Basis = DefaultBasis> {
    ptr: RelPtr<T, A::Region, B>,
    len: B::Usize,
    cap: B::Usize,
    alloc: A,
}

impl<T, A, B> DropRaw for RelVec<T, A, B>
where
    T: DropRaw,
    A: RawRegionalAllocator + DropRaw,
    B: Basis,
    <B as Basis>::Usize: DropRaw,
{
    #[inline]
    unsafe fn drop_raw(mut this: Mut<'_, Self>) {
        let layout = Layout::array::<T>(this.capacity()).unwrap();
        let inner = Self::deref_mut_raw(this.as_mut());

        let inner_ptr = inner.as_non_null();
        // SAFETY: The elements contained in the `RelVec` are always valid for
        // dropping. This drop call has the last reference to them, so they will
        // never be accessed again.
        unsafe {
            DropRaw::drop_raw(inner);
        }

        munge!(let RelVec { ptr, len, cap, alloc } = this);

        // SAFETY: `ptr` is never null and always allocated in `alloc` with a
        // layout of `layout`.
        unsafe {
            A::raw_deallocate(alloc.as_ref(), inner_ptr.cast(), layout);
        }

        // SAFETY: `ptr` and `alloc` are always valid for dropping and are not
        // accessed again.
        unsafe {
            DropRaw::drop_raw(ptr);
            DropRaw::drop_raw(len);
            DropRaw::drop_raw(cap);
            DropRaw::drop_raw(alloc);
        }
    }
}

impl<T, A: RawRegionalAllocator, B: Basis> RelVec<T, A, B> {
    /// Returns `true` if the `RelVec` contains no elements.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the number of elements in the `RelVec`.
    #[inline]
    pub fn len(&self) -> usize {
        B::to_native_usize(self.len).unwrap()
    }

    /// Returns a raw pointer to the `RelVec`'s buffer, or a dangling raw
    /// pointer valid for zero sized reads if the `RelVec` didn't allocate.
    #[inline]
    pub fn as_ptr(this: Ref<'_, Self>) -> *const T {
        munge!(let RelVec { ptr, .. } = this);

        // SAFETY: The relative pointer of a `RelVec` is never null.
        unsafe { RelPtr::as_ptr_unchecked(ptr) }
    }

    /// Returns an unsafe mutable pointer to the `RelVec`'s buffer, or a
    /// dangling raw pointer valid for zero sized reads if the `RelVec` didn't
    /// allocate.
    #[inline]
    pub fn as_mut_ptr(this: Mut<'_, Self>) -> *mut T {
        munge!(let RelVec { ptr, .. } = this);

        // SAFETY: The relative pointer of a `RelVec` is never null.
        unsafe { RelPtr::as_mut_ptr_unchecked(ptr) }
    }

    /// Forces the length of the vector to `new_len`.
    ///
    /// # Safety
    ///
    /// - `new_len` must be less than or equal to `capacity()`.
    /// - The elements at `old_len..new_len` must be initialized.
    pub unsafe fn set_len(this: Mut<'_, Self>, new_len: usize) {
        munge!(let RelVec { mut len, .. } = this);
        *len = B::from_native_usize(new_len).unwrap();
    }

    /// Returns the maximum number of elements the `RelVec` can contain before
    /// resizing.
    #[inline]
    pub fn capacity(&self) -> usize {
        B::to_native_usize(self.cap).unwrap()
    }

    /// Returns a reference to the underlying allocator.
    #[inline]
    pub fn allocator(this: Ref<'_, Self>) -> Ref<'_, A> {
        munge!(let RelVec { alloc, .. } = this);
        alloc
    }

    /// Returns a `Ref` to a slice of the elements in the `RelVec`.
    #[inline]
    pub fn as_slice(this: Ref<'_, Self>) -> Ref<'_, [T]> {
        DerefRaw::deref_raw(this)
    }

    /// # Safety
    ///
    /// `index` must be less than `capacity`.
    unsafe fn slot(
        this: Mut<'_, Self>,
        index: usize,
    ) -> In<Slot<'_, T>, A::Region> {
        munge!(let RelVec { ptr, .. } = this);
        // SAFETY: The `ptr` of a `RelVec` is always non-null.
        let ptr = unsafe { RelPtr::as_mut_ptr_unchecked(ptr) };
        // SAFETY: The `ptr` of a `RelVec` is always non-null, properly aligned,
        // and valid for reads and writes. Because `this` is mutably borrowed
        // for `'_`, the created reference cannot be aliased for `'_`.
        let slot = unsafe { Slot::new_unchecked(ptr.add(index)) };
        // SAFETY: All slots of the `RelVec` are allocated in `self.alloc`, and
        // since `A` implements `RawRegionalAllocator`, it guarantees that the
        // memory it allocates is located in its region.
        unsafe { In::new_unchecked(slot) }
    }

    /// # Safety
    ///
    /// `index` must be less than `len`. The returned `Val` may drop its
    /// contained value when it is dropped. Special care must be taken to ensure
    /// that this does not cause a dropped element to exist in the initialized
    /// section of the `RelVec.
    unsafe fn take(
        this: Mut<'_, Self>,
        index: usize,
    ) -> In<Val<'_, T>, A::Region>
    where
        T: DropRaw,
    {
        // SAFETY: The caller has guaranteed that `index` is less than `len`.
        let slot = unsafe { Self::slot(this, index) };
        // SAFETY: The slot at `index` is guaranteed to be initialized and valid
        // for dropping because `index < len`, and all elements of `RelVec` are
        // treated as pinned.
        let initialize = |s| unsafe { Val::from_slot_unchecked(s) };
        // SAFETY: `initialize` returns a `Val` of the given `Slot`, which is
        // always located in the same region as the `Slot` it is derived from.
        unsafe { In::map_unchecked(slot, initialize) }
    }

    /// Reserves capacity for at least `additional` more elements to be inserted
    /// in the given `RelVec<T>`. The collection may reserve more space to
    /// speculatively avoid frequent reallocations. After calling `reserve`, the
    /// capacity will be greater than or equal to `self.len() + additional`.
    /// Does nothing if capacity is already sufficient.
    ///
    /// # Panics
    ///
    /// Panics if the new capacity exceeds `isize::MAX` bytes.
    pub fn reserve(mut this: Mut<'_, Self>, additional: usize)
    where
        T: Move<A::Region>,
    {
        let min_cap = this.len() + additional;
        if min_cap > this.capacity() {
            let new_cap = min_cap.checked_next_power_of_two().unwrap();

            let old_layout = Layout::array::<T>(this.capacity()).unwrap();
            let new_layout = Layout::array::<T>(new_cap).unwrap();

            let ptr = Self::as_mut_ptr(this.as_mut());
            // SAFETY: The pointer of a `RelVec` is always non-null.
            let old_ptr = unsafe { ptr::NonNull::new_unchecked(ptr.cast()) };

            // SAFETY:
            // - `old_ptr` is the memory for the `RelVec`, which was allocated
            //   with `old_layout`.
            // - `new_layout` has a strictly larger size than `old_layout`
            //   because `new_cap` is greater than `min_cap`, which is greater
            //   than `this.capacity()`.
            let grew_in_place = unsafe {
                RawAllocator::raw_grow_in_place(
                    Self::allocator(this.as_ref()),
                    old_ptr,
                    old_layout,
                    new_layout,
                )
                .is_ok()
            };

            if !grew_in_place {
                let allocation = RawAllocator::raw_allocate(
                    Self::allocator(this.as_ref()),
                    new_layout,
                );
                let new_ptr = allocation.unwrap().as_ptr().cast::<T>();
                for i in 0..this.len() {
                    // SAFETY:
                    // - `new_ptr` is the pointer of a `NonNull`, so it must be
                    //   non-null. It is guaranteed to be aligned to
                    //   `new_layout.align()` by the implementation of
                    //   `RawAllocator`, which is at least `align_of::<T>()`. It
                    //   is also guaranteed to be valid for reads and writes of
                    //   at least `new_layout.size()` bytes, which covers every
                    //   element slot in `new_ptr`.
                    // - `new_ptr` is freshly-allocated, so only we have access
                    //   to it. It is not currently aliased by any other
                    //   pointers.
                    let out = unsafe { Slot::new_unchecked(new_ptr.add(i)) };
                    // SAFETY: `new_ptr` is allocated in `this.alloc`, and since
                    // `A` implements `RawRegionalAllocator`, it guarantees that
                    // memory it allocates is located in its region.
                    let out = unsafe { In::new_unchecked(out) };
                    // SAFETY: `i` is less than `len` and we move out of it then
                    // free the backing storage so it can't be accessed
                    // afterward.
                    let value = unsafe { Self::take(this.as_mut(), i) };
                    T::r#move(value, out);
                }

                munge!(let RelVec { ptr, alloc, .. } = this.as_mut());
                let new_ptr =
                    // SAFETY: `new_ptr` is allocated in `this.alloc`, and
                    // since `A` implements `RawRegionalAllocator` it guarantees
                    // that memory it allocates is located in its region.
                    unsafe { In::<_, A::Region>::new_unchecked(new_ptr) };
                RelPtr::set(ptr, new_ptr);

                // SAFETY:
                // - `old_ptr` is currently allocated because it was previously
                //   allocated and `grow_in_place` failed.
                // - `old_layout` was the layout used to allocate `old_ptr`.
                unsafe {
                    RawAllocator::raw_deallocate(
                        alloc.as_ref(),
                        old_ptr,
                        old_layout,
                    );
                }
            }

            munge!(let RelVec { mut cap, .. } = this);
            *cap = B::from_native_usize(new_cap).unwrap();
        }
    }

    /// Appends an element to the back of a collection.
    ///
    /// # Panics
    ///
    /// Panics if the new capacity exceeds `isize::MAX` bytes.
    pub fn push<E>(mut this: Mut<'_, Self>, value: E)
    where
        T: Move<A::Region>,
        E: Emplace<T, A::Region>,
    {
        Self::reserve(this.as_mut(), 1);
        let len = this.len();

        // SAFETY: `len` is definitely less than `capacity` because we reserved
        // one slot at the end of our storage and `len` is equal to the length
        // of the `RelVec`.
        let slot = unsafe { Self::slot(this.as_mut(), len) };
        value.emplace(slot);

        // SAFETY: `len + 1` must be less than or equal to `capacity` because we
        // reserved space for one additional element. We just initialized that
        // element by emplacing to it.
        unsafe {
            Self::set_len(this, len + 1);
        }
    }

    /// Extends the `RelVec` with the contents of an iterator.
    pub fn extend<I>(mut this: Mut<'_, Self>, mut values: I)
    where
        T: Move<A::Region>,
        I: Iterator,
        I::Item: Emplace<T, A::Region>,
    {
        // We can avoid setting our length every time we push a new element by
        // reserving the iterator's estimated size and pushing as many as we can
        // up to that limit. Then
        let reserve = values.size_hint().0;
        let mut new_len = this.len();

        Self::reserve(this.as_mut(), reserve);
        while new_len < this.capacity() {
            if let Some(value) = values.next() {
                // SAFETY: `len + i` must be less than `capacity` because we
                // reserved `reserve` slots at the end of our storage, `len`
                // is equal to the length of the `RelVec`, and `i` is less than
                // `reserve`.
                let slot = unsafe { Self::slot(this.as_mut(), new_len) };
                new_len += 1;
                value.emplace(slot);
            } else {
                break;
            }
        }

        // SAFETY: `len + reserve` must be less than or equal to `capacity`
        // because we reserved space for `reserve` additional elements. We just
        // initialized those element by emplacing to them.
        unsafe {
            Self::set_len(this.as_mut(), new_len);
        }

        // The `size_hint` from `values` isn't required to be accurate, so we
        // have to slowly push any remaining values.
        for value in values {
            Self::push(this.as_mut(), value);
        }
    }

    /// Clears the `RelVec`, removing all values.
    ///
    /// Note that this method has no effect on the allocated capacity of the
    /// `RelVec`.
    pub fn clear(mut this: Mut<'_, Self>)
    where
        T: DropRaw,
    {
        for i in 0..this.len() {
            // SAFETY: `i` is less than `this.len()` and we set the `len` to 0
            // so the removed element won't be accessed again.
            let val = unsafe { Self::take(this.as_mut(), i) };
            drop(val);
        }
        // SAFETY: 0 indicates that there are no initialized elements, and must
        // be less than or equal to the current capacity because capacity cannot
        // be less than 0.
        unsafe {
            Self::set_len(this, 0);
        }
    }
}

impl<T, A: RawRegionalAllocator, B: Basis> DerefRaw for RelVec<T, A, B> {
    type Target = [T];

    fn deref_raw(this: Ref<'_, Self>) -> Ref<'_, [T]> {
        let ptr = Self::as_ptr(this);
        let slice_ptr = ptr::slice_from_raw_parts(ptr, this.len());

        // SAFETY:
        // - We already ensured that `ptr` is never null and `slice_ptr` is just
        //   a copy of `ptr`. `ptr` is additionally always properly aligned and
        //   valid for reads.
        // - `this` is borrowed for `'_` so it cannot alias any other mutable
        //   references for `'_`.
        // - The relative pointer of a `RelVec` is always initialized.
        unsafe { Ref::new_unchecked(slice_ptr) }
    }
}

impl<T, A: RawRegionalAllocator, B: Basis> DerefMutRaw for RelVec<T, A, B> {
    fn deref_mut_raw(mut this: Mut<'_, Self>) -> Mut<'_, [T]> {
        let ptr = Self::as_mut_ptr(this.as_mut());
        let slice_ptr = ptr::slice_from_raw_parts_mut(ptr, this.len());

        // SAFETY:
        // - We already ensured that `ptr` is never null and `slice_ptr` is just
        //   a copy of `ptr`. `ptr` is additionally always properly aligned and
        //   valid for reads and writes.
        // - `this` is borrowed for `'_` so it cannot alias any other accessible
        //   references for `'_`.
        // - The relative pointer of a `RelVec` is always initialized and
        //   treated as immovable.
        unsafe { Mut::new_unchecked(slice_ptr) }
    }
}

impl<T, A, B> IndexRaw<usize> for RelVec<T, A, B>
where
    A: RawRegionalAllocator,
    B: Basis,
{
    type Output = T;

    fn index_raw(this: Ref<'_, Self>, index: usize) -> Ref<'_, Self::Output> {
        IndexRaw::index_raw(DerefRaw::deref_raw(this), index)
    }

    unsafe fn index_raw_unchecked(
        this: Ref<'_, Self>,
        index: usize,
    ) -> Ref<'_, Self::Output> {
        // SAFETY: The caller has guaranteed that `index` is in bounds for
        // indexing.
        unsafe {
            IndexRaw::index_raw_unchecked(DerefRaw::deref_raw(this), index)
        }
    }
}

impl<T, A, B> IndexMutRaw<usize> for RelVec<T, A, B>
where
    A: RawRegionalAllocator,
    B: Basis,
{
    fn index_mut_raw(
        this: Mut<'_, Self>,
        index: usize,
    ) -> Mut<'_, Self::Output> {
        IndexMutRaw::index_mut_raw(DerefMutRaw::deref_mut_raw(this), index)
    }

    unsafe fn index_mut_raw_unchecked(
        this: Mut<'_, Self>,
        index: usize,
    ) -> Mut<'_, Self::Output> {
        // SAFETY: The caller has guaranteed that `index` is in bounds for
        // indexing.
        unsafe {
            IndexMutRaw::index_mut_raw_unchecked(
                DerefMutRaw::deref_mut_raw(this),
                index,
            )
        }
    }
}

impl<T, A, B> DebugRaw for RelVec<T, A, B>
where
    T: DebugRaw,
    A: RawRegionalAllocator,
    B: Basis,
{
    fn fmt_raw(
        this: Ref<'_, Self>,
        f: &mut fmt::Formatter<'_>,
    ) -> Result<(), fmt::Error> {
        f.debug_list()
            .entries((0..this.len()).map(|i| IndexRaw::index_raw(this, i)))
            .finish()
    }
}

/// An emplacer for a new, empty `RelVec`.
pub struct New<R>(pub R);

// SAFETY:
// - `RelVec` is `Sized` and always has metadata `()`, so `emplaced_meta` always
//   returns valid metadata for it.
// - `emplace_unsized_unchecked` initializes its `out` parameter.
unsafe impl<T, A, B, R> Emplace<RelVec<T, A, B>, R::Region> for New<R>
where
    T: DropRaw,
    A: DropRaw + RawRegionalAllocator<Region = R::Region>,
    B: Basis,
    <B as Basis>::Usize: DropRaw,
    R: RelAllocator<A>,
{
    fn emplaced_meta(&self) -> <RelVec<T, A, B> as Pointee>::Metadata {}

    unsafe fn emplace_unsized_unchecked(
        self,
        out: In<Slot<'_, RelVec<T, A, B>>, R::Region>,
    ) {
        WithCapacity(self.0, 0).emplace(out);
    }
}

/// An emplacer for a new `RelVec` with an initial capacity.
pub struct WithCapacity<R>(pub R, pub usize);

// SAFETY:
// - `RelVec` is `Sized` and always has metadata `()`, so `emplaced_meta` always
//   returns valid metadata for it.
// - `emplace_unsized_unchecked` initializes its `out` parameter by emplacing
//   and writing to each field.
unsafe impl<T, A, B, R> Emplace<RelVec<T, A, B>, R::Region> for WithCapacity<R>
where
    T: DropRaw,
    A: DropRaw + RawRegionalAllocator<Region = R::Region>,
    B: Basis,
    <B as Basis>::Usize: DropRaw,
    R: RelAllocator<A>,
{
    fn emplaced_meta(
        &self,
    ) -> <RelVec<T, A, B> as ptr_meta::Pointee>::Metadata {
    }

    unsafe fn emplace_unsized_unchecked(
        self,
        out: In<Slot<'_, RelVec<T, A, B>>, R::Region>,
    ) {
        let Self(alloc, cap) = self;

        let ptr = alloc
            .allocate(Layout::array::<T>(cap).unwrap())
            .unwrap()
            .cast()
            .as_ptr();
        // SAFETY: The pointer returned from `allocate` is guaranteed to be in
        // the region of `R`.
        let ptr = unsafe { In::new_unchecked(ptr) };

        munge!(
            let RelVec {
                ptr: out_ptr,
                len: out_len,
                cap: out_cap,
                alloc: out_alloc,
            } = out;
        );

        ptr.emplace(out_ptr);
        In::into_inner(out_len).write(B::from_native_usize(0).unwrap());
        In::into_inner(out_cap).write(B::from_native_usize(cap).unwrap());
        alloc.emplace(out_alloc);
    }
}
