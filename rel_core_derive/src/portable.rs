use ::macroix::{
    repr::{BaseKind, Int, PrimitiveType, Repr},
    visit_fields,
    AttrValue,
};
use ::proc_macro2::{Span, TokenStream};
use ::quote::quote;
use ::syn::{parse2, parse_quote, Data, DeriveInput, Error, Path};

pub fn derive(mut input: DeriveInput) -> Result<TokenStream, Error> {
    let mut repr = None;
    let mut rel_core = None;
    for attr in input.attrs.iter() {
        if attr.path.is_ident("repr") {
            Repr::merge_attr(&mut repr, attr.tokens.clone())?;
        } else if attr.path.is_ident("rel_core") {
            rel_core =
                Some(parse2::<AttrValue<Path>>(attr.tokens.clone())?.value);
        }
    }
    let rel_core = rel_core.unwrap_or_else(|| parse_quote! { ::rel_core });

    let repr = repr.ok_or_else(|| {
        Error::new(
            Span::call_site(),
            "`Portable` types require an explicit `repr` attribute",
        )
    })?;
    let repr_base = repr.base.ok_or_else(|| {
        Error::new(
            Span::call_site(),
            "`Portable` types require a base `repr` kind",
        )
    })?;

    match input.data {
        Data::Struct(_) => {
            if !matches!(repr_base.kind, BaseKind::C | BaseKind::Transparent) {
                return Err(Error::new_spanned(
                    repr_base.kind_token,
                    "`Portable` structs must be `repr(C)` or \
                    `repr(transparent)`",
                ));
            }
        }
        Data::Enum(_) => match repr_base.kind {
            BaseKind::Primitive(Int::I8 | Int::U8) => (),
            BaseKind::C => match repr.primitive_type {
                Some(PrimitiveType {
                    int: Int::I8 | Int::U8,
                    ..
                }) => (),
                Some(PrimitiveType { int_token, .. }) => {
                    return Err(Error::new_spanned(
                        int_token,
                        "`Portable` enums that are `repr(C)` must have a \
                            primitive type of `i8` or `u8`",
                    ))
                }
                None => {
                    return Err(Error::new_spanned(
                        repr_base.kind_token,
                        "`Portable` enums that are `repr(C)` must specify \
                            a primitive type with `repr(C, i8)` or \
                            `repr(C, u8)`",
                    ))
                }
            },
            _ => {
                return Err(Error::new_spanned(
                    repr_base.kind_token,
                    "`Portable` enums must be `repr(i8)`, `repr(u8)`, \
                        `repr(C, i8)`, or `repr(C, u8)`",
                ));
            }
        },
        Data::Union(_) => {
            if !matches!(repr_base.kind, BaseKind::C | BaseKind::Transparent) {
                return Err(Error::new_spanned(
                    repr_base.kind_token,
                    "`Portable` unions must be `repr(C)` or \
                    `repr(transparent)`",
                ));
            }
        }
    }

    let where_clause = input.generics.make_where_clause();
    visit_fields(&input.data, |f| {
        let ty = &f.ty;
        where_clause
            .predicates
            .push(parse_quote! { #ty: #rel_core::Portable });
    });

    let (impl_generics, ty_generics, where_clause) =
        input.generics.split_for_impl();
    let ty_name = &input.ident;
    Ok(quote! {
        // SAFETY: This type has a valid `repr` and contains only `Portable`
        // fields.
        unsafe impl #impl_generics #rel_core::Portable
            for #ty_name #ty_generics #where_clause {}
    })
}
