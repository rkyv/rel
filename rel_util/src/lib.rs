//! Common utilities for working with `rel` types.

#![deny(
    missing_docs,
    unsafe_op_in_unsafe_fn,
    clippy::as_conversions,
    clippy::missing_safety_doc,
    clippy::undocumented_unsafe_blocks,
    rustdoc::broken_intra_doc_links,
    rustdoc::missing_crate_level_docs
)]
#![no_std]

use ::core::alloc::Layout;
use ::mischief::{Frame, Metadata};
use ::ptr_meta::Pointee;

/// A type that aligns its contents to 16-byte boundaries.
#[derive(Debug)]
#[repr(C, align(16))]
pub struct Align16<T: ?Sized>(pub T);

impl Align16<[u8]> {
    /// Returns a new [`Frame`] of at least the given size.
    pub fn frame(size: usize) -> Frame<Self> {
        let metadata = size.checked_add(15).unwrap() & !15;
        // SAFETY: `metadata` is the size of the slice contained in this
        // `Align16`, and is guaranteed to be a multiple of 16.
        unsafe { Frame::new_unsized(metadata) }
    }
}

impl Pointee for Align16<[u8]> {
    type Metadata = <[u8] as Pointee>::Metadata;
}

// SAFETY: `pointee_layout` returns the layout for a `u8` slice of length `self`
// aligned to 16 bytes.
unsafe impl Metadata<Align16<[u8]>> for usize {
    unsafe fn pointee_layout(self) -> Layout {
        // SAFETY: The caller has guaranteed that `self` is a valid length for
        // `Align16<[u8]>`, which always has an alignment of 16.
        unsafe { Layout::from_size_align_unchecked(self, 16) }
    }
}
