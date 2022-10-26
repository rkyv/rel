//! `situ` is a set of types and traits for handling types that cannot be moved.

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

pub mod alloc;
mod drop;
pub mod fmt;
mod r#mut;
pub mod ops;
mod owned_val;
mod pinned;
mod r#ref;
pub mod str;
mod val;

pub use self::{drop::*, owned_val::*, pinned::*, r#mut::*, r#ref::*, val::*};
