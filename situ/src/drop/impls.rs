use ::core::{
    marker::{PhantomData, PhantomPinned},
    mem::MaybeUninit,
};

use crate::{ops::IndexMutRaw, DropRaw, Mut};

macro_rules! impl_builtin {
    ($($ty:ty),*) => {
        $(
            // SAFETY:
            // - `emplaced_meta` returns `()`, the only valid metadata for
            //   `$ty`.
            // - `emplace_unsized_unchecked` initializes `out` by writing to it.
            impl DropRaw for $ty {
                #[inline]
                unsafe fn drop_raw(_: Mut<'_, Self>) {}
            }
        )*
    };
}

impl_builtin!(i8, u8, bool, (), PhantomPinned);

impl<T: DropRaw, const N: usize> DropRaw for [T; N] {
    #[inline]
    unsafe fn drop_raw(mut this: Mut<'_, Self>) {
        for i in 0..N {
            // SAFETY: The caller has guaranteed that `this` (and therefore all
            // of its elements) are valid for dropping. We drop each one exactly
            // once and never access them again.
            unsafe {
                DropRaw::drop_raw(IndexMutRaw::index_mut_raw(this.as_mut(), i));
            }
        }
    }
}

impl<T: DropRaw> DropRaw for [T] {
    #[inline]
    unsafe fn drop_raw(mut this: Mut<'_, Self>) {
        for i in 0..this.len() {
            // SAFETY: The caller has guaranteed that `this` (and therefore all
            // of its elements) are valid for dropping. We drop each one exactly
            // once and never access them again.
            unsafe {
                DropRaw::drop_raw(IndexMutRaw::index_mut_raw(this.as_mut(), i));
            }
        }
    }
}

impl<T: ?Sized> DropRaw for PhantomData<T> {
    #[inline]
    unsafe fn drop_raw(_: Mut<'_, Self>) {}
}

impl<T> DropRaw for MaybeUninit<T> {
    #[inline]
    unsafe fn drop_raw(_: Mut<'_, Self>) {}
}
