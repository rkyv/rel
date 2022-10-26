//! Procedural macros for `situ`.

#![deny(
    missing_docs,
    unsafe_op_in_unsafe_fn,
    clippy::as_conversions,
    clippy::missing_safety_doc,
    clippy::undocumented_unsafe_blocks,
    rustdoc::broken_intra_doc_links,
    rustdoc::missing_crate_level_docs
)]

mod drop_raw;

use ::proc_macro::TokenStream;
use ::syn::{parse_macro_input, DeriveInput};

/// Derives `DropRaw` on the annotated type.
#[proc_macro_derive(DropRaw, attributes(situ))]
pub fn derive_drop_raw(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    drop_raw::derive(derive_input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}
