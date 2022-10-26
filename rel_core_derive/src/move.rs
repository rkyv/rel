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

pub fn derive(input: DeriveInput) -> Result<TokenStream, Error> {
    let mut rel_core = None;
    let mut repr = None;
    for attr in input.attrs.iter() {
        if attr.path.is_ident("rel_core") {
            rel_core =
                Some(parse2::<AttrValue<Path>>(attr.tokens.clone())?.value);
        } else if attr.path.is_ident("repr") {
            Repr::merge_attr(&mut repr, attr.tokens.clone())?;
        }
    }
    let rel_core = rel_core.unwrap_or_else(|| parse_quote! { ::rel_core });

    let mut move_generics = input.generics.clone();
    move_generics.params.push(parse_quote! {
        __R: #rel_core::export::mischief::Region
    });
    let where_clause = move_generics.make_where_clause();
    where_clause.predicates.push(parse_quote! {
        Self: #rel_core::export::situ::DropRaw
    });
    visit_fields(&input.data, |f| {
        let ty = &f.ty;
        where_clause
            .predicates
            .push(parse_quote! { #ty: #rel_core::Move<__R> });
    });

    let (r#move, util) = match &input.data {
        Data::Enum(data_enum) => {
            let raw_enum = RawEnum::for_derive(&input)?;

            let raw_variants = &raw_enum.idents.variants;
            let raw_enum_fn = &raw_enum.idents.raw_enum_fn;
            let raw_discriminant_fn = &raw_enum.idents.discriminant_fn;
            let raw_variant_fn = &raw_enum.idents.variant_fn;

            let match_arms = data_enum.variants.iter().map(|v| {
                let ident = &v.ident;
                let move_variant = move_fields(&v.fields, &rel_core, true);
                quote! {
                    #raw_variants::#ident(this_ptr) => {
                        match #raw_variant_fn(out_raw) {
                            #raw_variants::#ident(out_ptr) => {
                                #move_variant
                            },
                            // SAFETY: `this` and `out` must be the same variant
                            // because we copied the discriminant from `this` to
                            // out.
                            _ => unsafe {
                                ::core::hint::unreachable_unchecked();
                            },
                        }
                    }
                }
            });

            (
                Some(quote! {
                    let this_raw = #raw_enum_fn(this_ptr);
                    let out_raw = #raw_enum_fn(out_ptr);
                    let this_discriminant = #raw_discriminant_fn(this_raw);
                    let out_discriminant = #raw_discriminant_fn(out_raw);
                    *out_discriminant = *this_discriminant;

                    match #raw_variant_fn(this_raw) {
                        #(#match_arms)*
                    }
                }),
                Some(raw_enum.tokens),
            )
        }
        Data::Struct(data_struct) => {
            (move_fields(&data_struct.fields, &rel_core, false), None)
        }
        Data::Union(data_union) => {
            return Err(Error::new_spanned(
                data_union.union_token,
                "`Move` cannot be derived for unions",
            ))
        }
    };

    let (impl_generics, _, where_clause) = move_generics.split_for_impl();
    let (_, ty_generics, _) = input.generics.split_for_impl();
    let ty_name = &input.ident;
    Ok(quote! {
        const _: () = {
            #util

            // SAFETY: `move_unsized_unchecked` initializes its `out` parameter
            // by destructuring it and moving all of the fields.
            #[allow(non_snake_case)]
            unsafe impl #impl_generics #rel_core::Move<__R>
                for #ty_name #ty_generics
            #where_clause
            {
                unsafe fn move_unsized_unchecked(
                    this: #rel_core::export::mischief::In<
                        #rel_core::export::situ::Val<'_, Self>,
                        __R,
                    >,
                    out: #rel_core::export::mischief::In<
                        #rel_core::export::mischief::Slot<'_, Self>,
                        __R,
                    >,
                ) {
                    let this_ptr = #rel_core::export::mischief::Pointer::target(
                        this.ptr(),
                    );
                    let out_ptr = #rel_core::export::mischief::Pointer::target(
                        out.ptr(),
                    );

                    ::core::mem::forget(this);

                    #r#move
                }
            }
        };
    })
}

fn move_field(rel_core: &Path) -> TokenStream {
    quote! {
        // SAFETY:
        // - `this_field` is a subfield of the value being moved out of, and so
        //   is guaranteed to be non-null, properly aligned, and valid for
        //   reading, writing, and dropping.
        // - The `Val` that `this_field` is derived from was forgotten, and
        //   `this_field` is the only pointer to the subfield we created, so it
        //   cannot alias any other accessible references for its lifetime.
        // - Because `this_field` is a subfield of the value being moved out of,
        //   and that value is initialized and immovable, the value pointed to
        //   by `this_field` is also initialized and immovable.
        let this_field = unsafe {
            #rel_core::export::situ::Val::new_unchecked(
                this_field
            )
        };
        // SAFETY:
        // - `out_field` is a pointer to a subfield of the slot being moved
        //   into, and so is guaranteed to be non-null, properly aligned, and
        //   valid for reads and writes.
        // - The `Slot` that `out_field` is derived from was forgotten, and
        //   `out_field` is the only pointer to the subfield we created, so it
        //   cannot alias any other accessible references for its lifetime.
        let out_field = unsafe {
            #rel_core::export::mischief::Slot::new_unchecked(
                out_field
            )
        };
        // SAFETY: `this_field` is a subfield of the value being moved out of,
        // so it must be contained in the same region as it.
        let this_field = unsafe {
            #rel_core::export::mischief::In::new_unchecked(
                this_field
            )
        };
        // SAFETY: `out_field` is a subfield of the value being moved out of, so
        // it must be contained in the same region as it.
        let out_field = unsafe {
            #rel_core::export::mischief::In::new_unchecked(
                out_field
            )
        };
        // SAFETY: The caller has guaranteed that the the `Val` and `Slot` being
        // moved have the same metadata, which means that all of their
        // corresponding fields must have the same metadata as each other.
        unsafe {
            #rel_core::MoveExt::r#move(this_field, out_field);
        }
    }
}

fn move_fields(
    fields: &Fields,
    rel_core: &Path,
    skip_discriminant: bool,
) -> Option<TokenStream> {
    match fields {
        Fields::Named(fields) => {
            let move_fields = fields.named.iter().map(|f| {
                let ident = &f.ident;
                let move_field = move_field(rel_core);
                quote! {
                    let this_field = ::core::ptr::addr_of_mut!(
                        (*this_ptr).#ident
                    );
                    let out_field = ::core::ptr::addr_of_mut!(
                        (*out_ptr).#ident
                    );
                    #move_field
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
                    let move_field = move_field(rel_core);
                    quote! {
                        let this_field = ::core::ptr::addr_of_mut!(
                            (*this_ptr).#i
                        );
                        let out_field = ::core::ptr::addr_of_mut!(
                            (*out_ptr).#i
                        );
                        #move_field
                    }
                });
            Some(quote! {
                #(#move_fields)*
            })
        }
        Fields::Unit => None,
    }
}
