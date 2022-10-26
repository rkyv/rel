#![deny(unsafe_op_in_unsafe_fn)]

use ::core::{
    alloc::Layout,
    cell::Cell,
    marker::{PhantomData, PhantomPinned},
    mem::forget,
    ptr::{slice_from_raw_parts_mut, NonNull},
};
use ::heresy::alloc::{AllocError, Allocator};
use ::mischief::{In, Region, RegionalAllocator, Singleton, Slot, Unique};
use ::munge::munge;
use ::ptr_meta::Pointee;
use ::rel_alloc::alloc::RelAllocator;
use ::rel_core::{
    Basis,
    DefaultBasis,
    Emplace,
    EmplaceExt,
    Move,
    MoveExt,
    Portable,
    RelRef,
};
use ::situ::{
    alloc::{RawAllocator, RawRegionalAllocator},
    DropRaw,
    OwnedVal,
    Pinned,
    Ref,
};

#[derive(Debug)]
pub struct SlabError;

#[derive(Portable, Unique)]
#[repr(C, align(8))]
struct SlabControl<U, B: Basis = DefaultBasis> {
    root: Cell<B::Usize>,
    len: Cell<B::Usize>,
    cap: Cell<B::Usize>,
    #[unique]
    unique: U,
    _pinned: PhantomPinned,
}

impl<U, B: Basis> SlabControl<U, B> {
    const LAYOUT: Layout = Layout::new::<Self>();

    fn root(&self) -> usize {
        B::to_native_usize(self.root.get()).unwrap()
    }

    fn cap(&self) -> usize {
        B::to_native_usize(self.cap.get()).unwrap()
    }

    fn len(&self) -> usize {
        B::to_native_usize(self.len.get()).unwrap()
    }

    fn allocate(
        this: Ref<'_, Self>,
        layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        if layout.align() > Self::LAYOUT.align() {
            Err(AllocError)
        } else {
            let start =
                (this.len() + layout.align() - 1) & !(layout.align() - 1);
            let available = this.cap() - start;
            if available < layout.size() {
                Err(AllocError)
            } else {
                this.len
                    .set(B::from_native_usize(start + layout.size()).unwrap());
                let address = unsafe { this.as_ptr().cast::<u8>().add(start) };
                let slice_ptr =
                    slice_from_raw_parts_mut(address, layout.size());
                Ok(unsafe { NonNull::new_unchecked(slice_ptr) })
            }
        }
    }

    fn try_new_in(
        bytes: Slot<'_, [u8]>,
        unique: U,
    ) -> Result<Ref<'_, Self>, SlabError> {
        let max_cap = bytes.len();
        let mut out = Self::try_cast_slot_from_bytes(bytes)?;

        munge!(
            let SlabControl {
                root: mut out_root,
                len: mut out_len,
                cap: mut out_cap,
                unique: mut out_unique,
                ..
            } = out.as_mut()
        );

        out_root.write(Cell::new(B::from_native_usize(0).unwrap()));
        out_len.write(Cell::new(
            B::from_native_usize(Self::LAYOUT.size()).unwrap(),
        ));
        out_cap.write(Cell::new(B::from_native_usize(max_cap).unwrap()));
        out_unique.write(unique);

        Ok(unsafe { Ref::new_unchecked(out.as_ptr()) })
    }

    fn try_from_bytes(
        bytes: Slot<'_, [u8]>,
        unique: U,
    ) -> Result<Ref<'_, Self>, SlabError> {
        let max_cap = bytes.len();
        let slot = Self::try_cast_slot_from_bytes(bytes)?;

        forget(unique);
        let result = unsafe { Ref::new_unchecked(slot.as_ptr()) };

        if result.len() > result.cap() || result.cap() > max_cap {
            Err(SlabError)
        } else {
            Ok(result)
        }
    }

    fn try_cast_slot_from_bytes(
        slot: Slot<'_, [u8]>,
    ) -> Result<Slot<'_, Self>, SlabError> {
        let len = slot.len();
        if len < Self::LAYOUT.size()
            || slot.as_ptr() as *mut u8 as usize & (Self::LAYOUT.align() - 1)
                != 0
        {
            Err(SlabError)
        } else {
            let slot = unsafe { slot.cast::<Self>() };
            Ok(slot)
        }
    }

    fn shrink_to_fit(&self) -> usize {
        let len = self.len.get();
        self.cap.set(len);
        Self::LAYOUT.size() + B::to_native_usize(len).unwrap()
    }

    unsafe fn deposit<T>(this: Ref<'_, Self>, val: *mut T) -> bool {
        let base = this.as_ptr() as usize;
        let target = val as usize;
        if this.root() == 0 && target >= base {
            this.root.set(B::from_native_usize(target - base).unwrap());
            true
        } else {
            false
        }
    }

    unsafe fn withdraw<T>(this: Ref<'_, Self>) -> Option<*mut T> {
        if this.root() != 0 {
            unsafe {
                Some(this.as_ptr().cast::<u8>().add(this.root()).cast::<T>())
            }
        } else {
            None
        }
    }
}

unsafe impl<U: Unique, B: Basis> Pinned<SlabRegion<U>> for SlabControl<U, B> {}

#[derive(Singleton)]
pub struct SlabRegion<U> {
    _phantom: PhantomData<U>,
}

unsafe impl<U: Unique> Region for SlabRegion<U> {}

#[derive(Singleton)]
pub struct SlabAllocator<'a, U, B: Basis = DefaultBasis> {
    inner: Ref<'a, SlabControl<U, B>>,
}

impl<'a, U, B: Basis> SlabAllocator<'a, U, B> {
    pub fn try_new_in(
        bytes: Slot<'a, [u8]>,
        unique: U,
    ) -> Result<Self, SlabError> {
        Ok(Self {
            inner: SlabControl::try_new_in(bytes, unique)?,
        })
    }

    pub fn try_from_bytes(
        bytes: Slot<'a, [u8]>,
        unique: U,
    ) -> Result<Self, SlabError> {
        Ok(Self {
            inner: SlabControl::try_from_bytes(bytes, unique)?,
        })
    }

    pub fn shrink_to_fit(&self) -> usize {
        self.inner.shrink_to_fit()
    }

    pub fn deposit<T>(
        &self,
        mut val: OwnedVal<T, Self>,
    ) -> Option<OwnedVal<T, Self>>
    where
        T: DropRaw + Portable,
        Self: Singleton,
    {
        let was_stored =
            unsafe { SlabControl::deposit(self.inner, val.as_mut().as_ptr()) };
        if was_stored {
            forget(val);
            None
        } else {
            Some(val)
        }
    }

    /// Withdraws a previously-deposited root object.
    ///
    /// # Safety
    ///
    /// The previously-deposited root object must be compatible with type `T`.
    pub unsafe fn withdraw_unchecked<T>(&self) -> Option<OwnedVal<T, Self>>
    where
        T: DropRaw + Portable,
    {
        let result = unsafe { SlabControl::withdraw(self.inner) };
        result.map(|ptr| unsafe { OwnedVal::from_raw_in(ptr, *self) })
    }
}

impl<U, B: Basis> Clone for SlabAllocator<'_, U, B> {
    fn clone(&self) -> Self {
        Self { inner: self.inner }
    }
}

impl<U, B: Basis> Copy for SlabAllocator<'_, U, B> {}

unsafe impl<U, B: Basis> Allocator for SlabAllocator<'_, U, B> {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        SlabControl::allocate(self.inner, layout)
    }

    unsafe fn deallocate(&self, _ptr: NonNull<u8>, _layout: Layout) {}
}

unsafe impl<'a, U, B1, B2>
    Emplace<RelSlabAllocator<'a, U, B1, B2>, SlabRegion<U>>
    for SlabAllocator<'a, U, B1>
where
    U: Unique,
    B1: Basis,
    B2: Basis,
{
    fn emplaced_meta(
        &self,
    ) -> <RelSlabAllocator<'a, U, B1, B2> as Pointee>::Metadata {
    }

    unsafe fn emplace_unsized_unchecked(
        self,
        out: In<Slot<'_, RelSlabAllocator<'a, U, B1, B2>>, SlabRegion<U>>,
    ) {
        munge!(let RelSlabAllocator { inner: out_inner } = out);
        In::new(self.inner).emplace(out_inner);
    }
}

unsafe impl<'a, U: Unique, B: Basis> RegionalAllocator
    for SlabAllocator<'a, U, B>
{
    type Region = SlabRegion<U>;
}

unsafe impl<'a, U, B1, B2> RelAllocator<RelSlabAllocator<'a, U, B1, B2>>
    for SlabAllocator<'a, U, B1>
where
    U: Unique,
    B1: Basis,
    B2: Basis,
{
}

#[derive(DropRaw, Portable)]
#[repr(C)]
pub struct RelSlabAllocator<
    'a,
    U: Unique,
    B1: Basis = DefaultBasis,
    B2: Basis = DefaultBasis,
> {
    inner: RelRef<'a, SlabControl<U, B1>, SlabRegion<U>, B2>,
}

unsafe impl<U, B1, B2> RawAllocator for RelSlabAllocator<'_, U, B1, B2>
where
    U: Unique,
    B1: Basis,
    B2: Basis,
{
    fn raw_allocate(
        this: Ref<'_, Self>,
        layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        munge!(let RelSlabAllocator { inner } = this);
        SlabControl::allocate(RelRef::deref(inner), layout)
    }

    unsafe fn raw_deallocate(
        _this: Ref<'_, Self>,
        _ptr: NonNull<u8>,
        _layout: Layout,
    ) {
    }
}

unsafe impl<'a, U, B1, B2> RawRegionalAllocator
    for RelSlabAllocator<'a, U, B1, B2>
where
    U: Unique,
    B1: Basis,
    B2: Basis,
{
    type Region = SlabRegion<U>;
}

unsafe impl<'a, U, B1, B2> Move<SlabRegion<U>>
    for RelSlabAllocator<'a, U, B1, B2>
where
    U: Unique,
    B1: Basis,
    B2: Basis,
{
    unsafe fn move_unsized_unchecked(
        this: In<situ::Val<'_, Self>, SlabRegion<U>>,
        out: In<Slot<'_, Self>, SlabRegion<U>>,
    ) {
        munge!(let RelSlabAllocator { inner } = this);
        munge!(let RelSlabAllocator { inner: out_inner } = out);

        MoveExt::r#move(inner, out_inner);
    }
}
