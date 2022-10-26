//! `rel_core` is a set of portable, relative-pointer based replacements for
//! core library types.

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

mod basis;
mod emplace;
pub mod export;
mod r#move;
pub mod option;
mod portable;
mod primitive;
pub mod rel_mem;
pub mod rel_ptr;
pub mod rel_ref;
pub mod rel_tuple;

pub use self::{
    basis::*,
    emplace::*,
    portable::*,
    primitive::*,
    r#move::*,
    rel_ptr::RelPtr,
    rel_ref::RelRef,
};
