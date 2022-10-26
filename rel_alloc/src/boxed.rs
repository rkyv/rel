//! A pointer type for heap allocation.

use ::core::{alloc::Layout, fmt, mem::MaybeUninit};
use ::mischief::{In, Slot};
use ::munge::munge;
use ::ptr_meta::Pointee;
use ::rel_core::{
    Basis,
    BasisPointee,
    DefaultBasis,
    Emplace,
    EmplaceExt,
    Move,
    Portable,
    RelPtr,
};
use ::situ::{
    alloc::RawRegionalAllocator,
    fmt::{DebugRaw, DisplayRaw},
    ops::{DerefMutRaw, DerefRaw, IndexMutRaw, IndexRaw},
    DropRaw,
    Mut,
    OwnedVal,
    Ref,
    Val,
};

use crate::alloc::RelAllocator;

/// A relative counterpart to `Box`.
#[derive(Move, Portable)]
#[repr(C)]
pub struct RelBox<
    T: BasisPointee<B> + ?Sized,
    A: RawRegionalAllocator,
    B: Basis = DefaultBasis,
> {
    ptr: RelPtr<T, A::Region, B>,
    alloc: A,
}

impl<T, A, B> DropRaw for RelBox<T, A, B>
where
    T: BasisPointee<B> + DropRaw + ?Sized,
    A: RawRegionalAllocator + DropRaw,
    B: Basis,
{
    #[inline]
    unsafe fn drop_raw(mut this: Mut<'_, Self>) {
        let inner = Self::deref_mut_raw(this.as_mut());
        let layout = Layout::for_value(&*inner);

        let inner_ptr = inner.as_non_null();
        // SAFETY: `inner` is valid for dropping because we own it. It will
        // never be accessed again because `this` will never be accessed again.
        unsafe {
            DropRaw::drop_raw(inner);
        }

        munge!(let RelBox { ptr, alloc } = this);

        // SAFETY: `ptr` is never null and always allocated in `alloc` with a
        // layout of `layout`.
        unsafe {
            A::raw_deallocate(alloc.as_ref(), inner_ptr.cast(), layout);
        }

        // SAFETY: `ptr` and `alloc` are always valid for dropping and are not
        // accessed again.
        unsafe {
            DropRaw::drop_raw(ptr);
            DropRaw::drop_raw(alloc);
        }
    }
}

impl<T, A, B> RelBox<T, A, B>
where
    T: BasisPointee<B> + ?Sized,
    A: RawRegionalAllocator,
    B: Basis,
{
    /// Returns a reference to the underlying allocator.
    #[inline]
    pub fn allocator(this: Ref<'_, Self>) -> Ref<'_, A> {
        munge!(let RelBox { alloc, .. } = this);
        alloc
    }
}

impl<T, A, B> DerefRaw for RelBox<T, A, B>
where
    T: BasisPointee<B> + ?Sized,
    A: RawRegionalAllocator,
    B: Basis,
{
    type Target = T;

    fn deref_raw(this: Ref<'_, Self>) -> Ref<'_, T> {
        munge!(let RelBox { ptr, .. } = this);
        // SAFETY:
        // - The value pointed to by a `RelBox` is always non-null,
        //   properly-aligned, and valid for reads.
        // - Because `this` is a shared reference and `ptr` is borrowed from it,
        //   `ptr` cannot alias any other mutable references for `'_`.
        // - The value pointed to by a `RelBox` is always initialized.
        unsafe { RelPtr::as_ref(ptr) }
    }
}

impl<T, A, B> DerefMutRaw for RelBox<T, A, B>
where
    T: BasisPointee<B> + ?Sized,
    A: RawRegionalAllocator,
    B: Basis,
{
    fn deref_mut_raw(this: Mut<'_, Self>) -> Mut<'_, T> {
        munge!(let RelBox { ptr, .. } = this);
        // SAFETY:
        // - The value pointed to by a `RelBox` is always non-null,
        //   properly-aligned, and valid for reads and writes.
        // - Because `this` is a shared reference and `ptr` is borrowed from it,
        //   `ptr` cannot alias any other accessible references for `'_`.
        // - The value pointed to by a `RelBox` is always initialized and
        //   treated as immovable.
        unsafe { RelPtr::as_mut(ptr) }
    }
}

impl<T, A, B> DebugRaw for RelBox<T, A, B>
where
    T: BasisPointee<B> + DebugRaw + ?Sized,
    A: RawRegionalAllocator,
    B: Basis,
{
    fn fmt_raw(
        this: Ref<'_, Self>,
        f: &mut fmt::Formatter<'_>,
    ) -> Result<(), fmt::Error> {
        DebugRaw::fmt_raw(DerefRaw::deref_raw(this), f)
    }
}

impl<T, A, B> DisplayRaw for RelBox<T, A, B>
where
    T: BasisPointee<B> + DisplayRaw + ?Sized,
    A: RawRegionalAllocator,
    B: Basis,
{
    fn fmt_raw(
        this: Ref<'_, Self>,
        f: &mut fmt::Formatter<'_>,
    ) -> Result<(), fmt::Error> {
        DisplayRaw::fmt_raw(DerefRaw::deref_raw(this), f)
    }
}

impl<T, A, B, Idx> IndexRaw<Idx> for RelBox<T, A, B>
where
    T: BasisPointee<B> + IndexRaw<Idx> + ?Sized,
    A: RawRegionalAllocator,
    B: Basis,
{
    type Output = <T as IndexRaw<Idx>>::Output;

    fn index_raw(this: Ref<'_, Self>, index: Idx) -> Ref<'_, Self::Output> {
        IndexRaw::index_raw(DerefRaw::deref_raw(this), index)
    }

    unsafe fn index_raw_unchecked(
        this: Ref<'_, Self>,
        index: Idx,
    ) -> Ref<'_, Self::Output> {
        // SAFETY: The caller has guaranteed that `index` is in bounds for
        // indexing.
        unsafe {
            IndexRaw::index_raw_unchecked(DerefRaw::deref_raw(this), index)
        }
    }
}

impl<T, A, B, Idx> IndexMutRaw<Idx> for RelBox<T, A, B>
where
    T: BasisPointee<B> + IndexMutRaw<Idx> + ?Sized,
    A: RawRegionalAllocator,
    B: Basis,
{
    fn index_mut_raw(this: Mut<'_, Self>, index: Idx) -> Mut<'_, Self::Output> {
        IndexMutRaw::index_mut_raw(DerefMutRaw::deref_mut_raw(this), index)
    }

    unsafe fn index_mut_raw_unchecked(
        this: Mut<'_, Self>,
        index: Idx,
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

impl<T, A, B> RelBox<MaybeUninit<T>, A, B>
where
    T: DropRaw,
    A: RawRegionalAllocator + DropRaw,
    B: Basis,
{
    /// Converts to a `RelBox<T, A, B>`.
    ///
    /// # Safety
    ///
    /// The value in this `RelBox` must be initialized.
    pub unsafe fn assume_init(b: Val<Self>) -> Val<RelBox<T, A, B>> {
        // SAFETY: The caller has guaranteed that the underlying `MaybeUninit`
        // has been properly initialized, and `MaybeUninit<T>` has the same
        // layout as the `T` it wraps.
        unsafe { b.cast() }
    }
}

// SAFETY:
// - `RelBox` is `Sized` and always has metadata `()`, so `emplaced_meta` always
//   returns valid metadata for it.
// - `emplace_unsized_unchecked` initializes its `out` parameter.
unsafe impl<T, A, B, R> Emplace<RelBox<T, A, B>, R::Region> for OwnedVal<T, R>
where
    T: BasisPointee<B> + DropRaw + ?Sized,
    A: DropRaw + RawRegionalAllocator<Region = R::Region>,
    B: Basis,
    R: RelAllocator<A>,
{
    #[inline]
    fn emplaced_meta(&self) -> <RelBox<T, A, B> as Pointee>::Metadata {}

    unsafe fn emplace_unsized_unchecked(
        self,
        mut out: In<Slot<'_, RelBox<T, A, B>>, R::Region>,
    ) {
        let this = In::new(self);
        let ptr = this.as_raw();
        let (_, alloc) = OwnedVal::into_raw_parts(In::into_inner(this));

        munge!(let RelBox { ptr: out_ptr, alloc: out_alloc } = out.as_mut());

        ptr.emplace(out_ptr);
        alloc.emplace(out_alloc);
    }
}
