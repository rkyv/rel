//! Procedural macros for `rel_core`.

#![deny(
    missing_docs,
    unsafe_op_in_unsafe_fn,
    clippy::as_conversions,
    clippy::missing_safety_doc,
    clippy::undocumented_unsafe_blocks,
    rustdoc::broken_intra_doc_links,
    rustdoc::missing_crate_level_docs
)]

mod r#move;
mod portable;

use ::proc_macro::TokenStream;
use ::syn::{parse_macro_input, DeriveInput};

/// Derives `Move` on the annotated type.
#[proc_macro_derive(Move, attributes(rel_core))]
pub fn derive_move(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    r#move::derive(derive_input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

/// Derives `Portable` on the annotated type.
#[proc_macro_derive(Portable, attributes(rel_core))]
pub fn derive_portable(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    portable::derive(derive_input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}
