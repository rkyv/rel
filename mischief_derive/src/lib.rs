//! Procedural macros for `mischief`.

#![deny(
    missing_docs,
    unsafe_op_in_unsafe_fn,
    clippy::as_conversions,
    clippy::missing_safety_doc,
    clippy::undocumented_unsafe_blocks,
    rustdoc::broken_intra_doc_links,
    rustdoc::missing_crate_level_docs
)]

mod singleton;
mod unique;

use ::proc_macro::TokenStream;
use ::syn::{parse_macro_input, DeriveInput};

/// Derives `Singleton` on the annotated type.
#[proc_macro_derive(Singleton, attributes(mischief))]
pub fn derive_singleton(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    singleton::derive(derive_input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

/// Derives `Unique` on the annotated type.
#[proc_macro_derive(Unique, attributes(mischief, unique))]
pub fn derive_unique(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    unique::derive(derive_input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}
