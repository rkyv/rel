use ::core::marker::PhantomData;
use ::mischief::{In, Region, Slot};
use ::situ::{ops::IndexMutRaw, Mut, Val};

use crate::{Move, MoveExt};

macro_rules! impl_builtin {
    ($($ty:ty),*) => {
        $(
            // SAFETY: `move_unsized_unchecked` initializes `out` by emplacing
            // to it.
            unsafe impl<R: Region> Move<R> for $ty {
                unsafe fn move_unsized_unchecked(
                    this: In<Val<'_, Self>, R>,
                    out: In<Slot<'_, Self>, R>,
                ) {
                    // SAFETY: `$ty` is `Sized`, so it has metadata `()` and
                    // `out` must have the same metadata as it.
                    unsafe {
                        Val::read_unsized_unchecked(
                            In::into_inner(this),
                            In::into_inner(out),
                        );
                    }
                }
            }
        )*
    };
}

impl_builtin!(i8, u8, bool, ());

// SAFETY: `move_unsized_unchecked` initializes its `out` parameter by emplacing
// to every element in it.
unsafe impl<T, R: Region, const N: usize> Move<R> for [T; N]
where
    T: Move<R>,
{
    unsafe fn move_unsized_unchecked(
        this: In<Val<'_, Self>, R>,
        out: In<Slot<'_, Self>, R>,
    ) {
        let mut this = Val::leak(In::into_inner(this));
        let mut out = In::into_inner(out);

        for i in 0..N {
            // SAFETY: `i` is in bounds because it must be less than the length
            // of the array, `N`.
            let this_i = unsafe {
                IndexMutRaw::index_mut_raw_unchecked(this.as_mut(), i)
            };
            // SAFETY:
            // - `this_i` is an element of `this`, which we own and may drop.
            // - `this_i` is only taken once, and `this` is leaked so its
            //   elements will not be accessed again.
            let this_i = unsafe { Mut::take(this_i) };
            // SAFETY: `this_i` is an element of `this`, which is located in
            // `R`, so `this_i` must also be located in `R`.
            let this_i = unsafe { In::new_unchecked(this_i) };
            // SAFETY: `i` is in bounds because it must be less than the length
            // of the array, `N`.
            let out_i = unsafe { out.as_mut().get_unchecked(i) };
            // SAFETY: `out_i` is an element of `out`, which is located in `R`,
            // so `out_i` must also be located in `R`.
            let out_i = unsafe { In::new_unchecked(out_i) };
            T::r#move(this_i, out_i);
        }
    }
}

// SAFETY: `move_unsized_unchecked` initializes its `out` parameter by emplacing
// to every element in it.
unsafe impl<T, R: Region> Move<R> for [T]
where
    T: Move<R>,
{
    unsafe fn move_unsized_unchecked(
        this: In<Val<'_, Self>, R>,
        out: In<Slot<'_, Self>, R>,
    ) {
        let len = this.len();
        let mut this = Val::leak(In::into_inner(this));
        let mut out = In::into_inner(out);

        for i in 0..len {
            // SAFETY: `i` is in bounds because it must be less than the length
            // of the array, `len`.
            let this_i = unsafe {
                IndexMutRaw::index_mut_raw_unchecked(this.as_mut(), i)
            };
            // SAFETY:
            // - `this_i` is an element of `this`, which we own and may drop.
            // - `this_i` is only taken once, and `this` is leaked so its
            //   elements will not be accessed again.
            let this_i = unsafe { Mut::take(this_i) };
            // SAFETY: `this_i` is an element of `this`, which is located in
            // `R`, so `this_i` must also be located in `R`.
            let this_i = unsafe { In::new_unchecked(this_i) };
            // SAFETY: `i` is in bounds because it must be less than the length
            // of the array, `len`.
            let out_i = unsafe { out.as_mut().get_unchecked(i) };
            // SAFETY: `out_i` is an element of `out`, which is located in `R`,
            // so `out_i` must also be located in `R`.
            let out_i = unsafe { In::new_unchecked(out_i) };
            T::r#move(this_i, out_i);
        }
    }
}

// SAFETY: `move_unsized_unchecked` does not have to initialize `out` because
// `PhantomData` is zero-sized and so always initialized.
unsafe impl<T: ?Sized, R: Region> Move<R> for PhantomData<T> {
    unsafe fn move_unsized_unchecked(
        _: In<Val<'_, Self>, R>,
        _: In<Slot<'_, Self>, R>,
    ) {
    }
}
