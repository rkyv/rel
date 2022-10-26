use ::macroix::{visit_fields, AttrValue};
use ::proc_macro2::TokenStream;
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
    visit_fields(&input.data, |f| {
        let ty = &f.ty;
        where_clause
            .predicates
            .push(parse_quote! { #ty: #mischief::Singleton });
    });

    let (impl_generics, ty_generics, where_clause) =
        input.generics.split_for_impl();
    let ty_name = &input.ident;
    Ok(quote! {
        unsafe impl #impl_generics #mischief::Singleton
            for #ty_name #ty_generics #where_clause {}
    })
}
