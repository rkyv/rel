//! An attribute proc macro that generates the output from `raw_enum` into the
//! scope of the annotated enum.

#![deny(
    missing_docs,
    unsafe_op_in_unsafe_fn,
    clippy::as_conversions,
    clippy::missing_safety_doc,
    clippy::undocumented_unsafe_blocks,
    rustdoc::broken_intra_doc_links,
    rustdoc::missing_crate_level_docs
)]

use ::proc_macro2::TokenStream;
use ::quote::ToTokens;
use ::raw_enum::RawEnum;
use ::syn::{parse_macro_input, DeriveInput, Error};

/// Generates a raw version of an enum.
#[proc_macro_attribute]
pub fn raw_enum(
    _attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    generate_raw_enum(&input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

fn generate_raw_enum(input: &DeriveInput) -> Result<TokenStream, Error> {
    let raw_enum = RawEnum::for_derive(input)?;

    let mut result = input.to_token_stream();
    result.extend(raw_enum.tokens);
    Ok(result)
}
