//! `rel_alloc` is a set of portable, relative-pointer based replacements for
//! alloc library types.

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
pub mod boxed;
mod emplace_in;
pub mod string;
pub mod vec;

pub use self::{
    boxed::RelBox,
    emplace_in::EmplaceIn,
    string::RelString,
    vec::RelVec,
};
