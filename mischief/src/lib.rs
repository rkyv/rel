//! Some extra mischief compatible with the standard library or `heresy`.

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

mod frame;
mod r#in;
mod metadata;
mod pointer;
mod region;
mod slot;
mod unique;

pub use self::{
    frame::*,
    metadata::*,
    pointer::*,
    r#in::*,
    region::*,
    slot::*,
    unique::*,
};
