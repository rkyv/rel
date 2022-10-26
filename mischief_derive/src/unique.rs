use ::macroix::{visit_fields, AttrValue};
use ::proc_macro2::{Span, TokenStream};
use ::quote::quote;
use ::syn::{parse2, parse_quote, DeriveInput, Error, Path};

pub fn derive(mut input: DeriveInput) -> Result<TokenStream, Error> {
    let mut mischief = None;
    for attr in input.attrs.iter() {
        if attr.path.is_ident("mischief") {
            mischief =
                Some(parse2::<AttrValue<Path>>(attr.tokens.clone())?.value);
        }
    }

    let mischief = mischief.unwrap_or_else(|| parse_quote! { ::mischief });

    let where_clause = input.generics.make_where_clause();
    let mut has_unique = false;
    visit_fields(&input.data, |f| {
        if f.attrs.iter().any(|a| a.path.is_ident("unique")) {
            has_unique = true;
            let ty = &f.ty;
            where_clause
                .predicates
                .push(parse_quote! { #ty: #mischief::Unique });
        }
    });
    if !has_unique {
        return Err(Error::new(
            Span::call_site(),
            "expected a field to be annotated with `#[unique]`",
        ));
    }

    let (impl_generics, ty_generics, where_clause) =
        input.generics.split_for_impl();
    let ty_name = &input.ident;
    Ok(quote! {
        unsafe impl #impl_generics #mischief::Unique
            for #ty_name #ty_generics #where_clause {}
    })
}
