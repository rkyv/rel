use ::macroix::{repr::Repr, visit_fields, AttrValue};
use ::proc_macro2::TokenStream;
use ::quote::quote;
use ::raw_enum::RawEnum;
use ::syn::{
    parse2,
    parse_quote,
    Data,
    DeriveInput,
    Error,
    Fields,
    Index,
    Path,
};

pub fn derive(mut input: DeriveInput) -> Result<TokenStream, Error> {
    let mut repr = None;
    let mut situ = None;
    for attr in input.attrs.iter() {
        if attr.path.is_ident("repr") {
            Repr::merge_attr(&mut repr, attr.tokens.clone())?;
        } else if attr.path.is_ident("situ") {
            situ = Some(parse2::<AttrValue<Path>>(attr.tokens.clone())?.value);
        }
    }
    let situ = situ.unwrap_or_else(|| parse_quote! { ::situ });

    let name = &input.ident;

    let where_clause = input.generics.make_where_clause();
    visit_fields(&input.data, |f| {
        let ty = &f.ty;
        where_clause
            .predicates
            .push(parse_quote! { #ty: #situ::DropRaw });
    });

    let (drop_raw, util) = match &input.data {
        Data::Enum(data_enum) => {
            let raw_enum = RawEnum::for_derive(&input)?;

            let raw_variants = &raw_enum.idents.variants;
            let raw_enum_fn = &raw_enum.idents.raw_enum_fn;
            let raw_variant_fn = &raw_enum.idents.variant_fn;

            let match_arms = data_enum.variants.iter().map(|v| {
                let ident = &v.ident;
                let drop_raw_variant = drop_raw_fields(&v.fields, &situ, true);
                quote! {
                    #raw_variants::#ident(this_ptr) => {
                        #drop_raw_variant
                    }
                }
            });

            (
                Some(quote! {
                    let this_raw = #raw_enum_fn(this_ptr);
                    match #raw_variant_fn(this_raw) {
                        #(#match_arms)*
                    }
                }),
                Some(raw_enum.tokens),
            )
        }
        Data::Struct(data_struct) => {
            (drop_raw_fields(&data_struct.fields, &situ, false), None)
        }
        Data::Union(data_union) => {
            return Err(Error::new_spanned(
                data_union.union_token,
                "`DropRaw` cannot be derived for unions",
            ))
        }
    };

    let (impl_generics, ty_generics, where_clause) =
        input.generics.split_for_impl();

    Ok(quote! {
        const _: () = {
            #util

            impl #impl_generics #situ::DropRaw for #name #ty_generics
            #where_clause
            {
                unsafe fn drop_raw(mut this: #situ::Mut<'_, Self>) {
                    let this_ptr = #situ::Mut::as_ptr(&this);
                    #drop_raw
                }
            }
        };
    })
}

fn drop_raw_fields(
    fields: &Fields,
    situ: &Path,
    skip_discriminant: bool,
) -> Option<TokenStream> {
    match fields {
        Fields::Named(fields) => {
            let move_fields = fields.named.iter().map(|f| {
                let ty = &f.ty;
                let ident = &f.ident;
                quote! {
                    <#ty as #situ::DropRaw>::drop_raw(
                        #situ::Mut::new_unchecked(
                            ::core::ptr::addr_of_mut!((*this_ptr).#ident)
                        )
                    );
                }
            });
            Some(quote! {
                #(#move_fields)*
            })
        }
        Fields::Unnamed(fields) => {
            let move_fields =
                fields.unnamed.iter().enumerate().map(|(i, _)| {
                    // In enum tuple structs, the tag is the first element so we
                    // have to skip over it.
                    let offset = if skip_discriminant { 1 } else { 0 };
                    let i = Index::from(i + offset);
                    quote! {
                        #situ::DropRaw::drop_raw(
                            #situ::Mut::new_unchecked(
                                ::core::ptr::addr_of_mut!((*this_ptr).#i)
                            )
                        );
                    }
                });
            Some(quote! {
                #(#move_fields)*
            })
        }
        Fields::Unit => None,
    }
}
