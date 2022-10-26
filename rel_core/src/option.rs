//! A value that may or may not exist.

use ::core::{hint::unreachable_unchecked, ptr::addr_of_mut};
use ::mischief::{In, Region, Slot};
use ::ptr_meta::Pointee;
use ::raw_enum_macro::raw_enum;
use ::situ::DropRaw;

use crate::{Emplace, EmplaceExt, Move, Portable};

/// A relative counterpart to `Option`.
#[derive(DropRaw, Move, Portable)]
#[rel_core = "crate"]
#[repr(u8)]
#[raw_enum]
pub enum RelOption<T> {
    /// No value.
    None,
    /// Some value of type `T`.
    Some(T),
}

// SAFETY:
// - `emplaced_meta` returns `()`, the only valid metadata for `Sized` types.
// - `emplace_unsized_unchecked` initializes its `out` parameter by always
//   setting the discriminant and additionally emplacing a value for the `Some`
//   variant.
unsafe impl<T: DropRaw, E, R: Region> Emplace<RelOption<T>, R> for Option<E>
where
    E: Emplace<T, R>,
{
    fn emplaced_meta(&self) -> <RelOption<T> as Pointee>::Metadata {}

    unsafe fn emplace_unsized_unchecked(
        self,
        out: In<Slot<'_, RelOption<T>>, R>,
    ) {
        let raw_out = raw_rel_option(out.ptr().as_ptr());
        let out_discriminant = raw_rel_option_discriminant(raw_out);

        match self {
            // SAFETY: `raw_rel_option_discriminant` guarantees that the pointer
            // it returns is properly aligned and valid for writes.
            None => unsafe {
                out_discriminant.write(RawRelOptionDiscriminant::None);
            },
            Some(emplacer) => {
                // SAFETY: `raw_rel_option_discriminant` guarantees that the
                // pointer it returns is properly aligned and valid for writes.
                unsafe {
                    out_discriminant.write(RawRelOptionDiscriminant::Some);
                }
                match raw_rel_option_variant(raw_out) {
                    RawRelOptionVariants::Some(out_ptr) => {
                        let value_ptr = addr_of_mut!((*out_ptr).1);
                        // SAFETY:
                        // - `value_ptr` is a pointer into `out`, so it is
                        //   non-null, properly aligned, and valid for reads and
                        //   writes.
                        // - `value_ptr` is a disjoint borrow of `out`, which is
                        //   guaranteed not to alias any other accessible
                        //   references, so the returned `Slot` will not either.
                        let slot = unsafe { Slot::new_unchecked(value_ptr) };
                        // SAFETY: `value_ptr` is a pointer into `out`, which is
                        // contained in `R`, so `value_ptr` must be contained in
                        // `R` as well.
                        let slot = unsafe { In::new_unchecked(slot) };
                        emplacer.emplace(slot);
                    }
                    // SAFETY: We wrote the `Some` discriminant to
                    // `out_discriminant` so it must be the `Some` variant.
                    _ => unsafe { unreachable_unchecked() },
                }
            }
        }
    }
}
