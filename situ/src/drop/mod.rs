mod impls;

pub use ::situ_derive::DropRaw;

use crate::Mut;

/// A type that can be dropped through a raw pointer, without creating an
/// intermediary reference.
///
/// This is different from `ptr::drop_in_place` because it does not invoke
/// `Drop::drop` with a mutable reference. `DropRaw::drop_raw` is the last
/// method called with access to the value; effectively a raw version of `Drop`.
/// This is to avoid creating an intermediate reference, which narrows
/// provenance for the dropped value.
///
/// Types that implement `Copy` may have their raw drops elided.
pub trait DropRaw {
    /// Drops the value pointed to by `this`.
    ///
    /// # Safety
    ///
    /// - The value pointed to by `this` must be valid for dropping.
    /// - After calling `drop_raw`, the value pointed to by `this` must never be
    ///   accessed again.
    unsafe fn drop_raw(this: Mut<'_, Self>);
}
