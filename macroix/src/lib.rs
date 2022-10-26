//! Common parsers and code generation for proc macros.

#![deny(
    missing_docs,
    unsafe_op_in_unsafe_fn,
    clippy::as_conversions,
    clippy::missing_safety_doc,
    clippy::undocumented_unsafe_blocks,
    rustdoc::broken_intra_doc_links,
    rustdoc::missing_crate_level_docs
)]

mod attr_value;
pub mod case;
pub mod repr;

use ::syn::{Data, Field, Fields, GenericParam, Generics, TypeParam};

pub use self::attr_value::*;

/// Visits each field of the given struct, enum, or union and calls a function
/// on each one.
pub fn visit_fields<F: FnMut(&Field)>(data: &Data, mut f: F) {
    match data {
        Data::Struct(data) => {
            visit_fields_inner(&data.fields, f);
        }
        Data::Enum(data) => {
            for variant in &data.variants {
                visit_fields_inner(&variant.fields, &mut f);
            }
        }
        Data::Union(data) => {
            for field in data.fields.named.iter() {
                f(field);
            }
        }
    }
}

fn visit_fields_inner<F: FnMut(&Field)>(fields: &Fields, mut f: F) {
    match fields {
        Fields::Named(fields) => {
            for field in fields.named.iter() {
                f(field);
            }
        }
        Fields::Unnamed(fields) => {
            for field in fields.unnamed.iter() {
                f(field);
            }
        }
        Fields::Unit => (),
    }
}

/// Inserts a type parameter at the correct position in some `Generics`.
pub fn insert_type_param(generics: &mut Generics, param: TypeParam) {
    let first_non_lifetime = generics
        .params
        .iter()
        .position(|p| !matches!(p, GenericParam::Lifetime(_)))
        .unwrap_or(generics.params.len());
    generics
        .params
        .insert(first_non_lifetime, GenericParam::Type(param));
}
