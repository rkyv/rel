//! A reformulation of the allocator API for more general-purpose use.

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

#[cfg(feature = "alloc")]
extern crate alloc as builtin_alloc;

pub mod alloc;
pub mod boxed;

pub use self::boxed::*;
