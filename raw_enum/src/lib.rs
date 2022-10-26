//! Generates a "raw" version of an enum comprised entirely of structs and
//! unions.

#![deny(
    missing_docs,
    unsafe_op_in_unsafe_fn,
    clippy::as_conversions,
    clippy::missing_safety_doc,
    clippy::undocumented_unsafe_blocks,
    rustdoc::broken_intra_doc_links,
    rustdoc::missing_crate_level_docs
)]

use ::macroix::{
    case::pascal_to_snake,
    repr::{BaseKind, Int, PrimitiveType, Repr},
};
use ::proc_macro2::{Span, TokenStream};
use ::quote::quote;
use ::syn::{
    Data,
    DataEnum,
    DeriveInput,
    Error,
    Fields,
    Generics,
    Ident,
    Variant,
};

/// The identifiers of the generated items.
pub struct RawIdents {
    /// The identifier of the raw enum.
    pub r#enum: Ident,
    /// The identifier of the raw enum discriminant.
    pub discriminant: Ident,
    /// The identifier of the raw enum fields union (if any).
    pub fields: Ident,
    /// The identifier of the variants enum.
    pub variants: Ident,
    variant_prefix: String,
    /// The identifier of the function that converts a pointer to an enum to a
    /// pointer to a raw enum.
    pub raw_enum_fn: Ident,
    /// The identifier of the function that gets a pointer to the discriminant
    /// of the raw enum.
    pub discriminant_fn: Ident,
    /// The identifier of the function that determines the variant of the enum
    /// and casts it to the appropriate variant struct.
    pub variant_fn: Ident,
}

impl RawIdents {
    fn for_enum(ident: &Ident) -> Self {
        let snake_case = pascal_to_snake(&ident.to_string());
        Self {
            r#enum: Ident::new(&format!("Raw{ident}Enum"), ident.span()),
            discriminant: Ident::new(
                &format!("Raw{ident}Discriminant"),
                ident.span(),
            ),
            fields: Ident::new(&format!("Raw{ident}Fields"), ident.span()),
            variants: Ident::new(&format!("Raw{ident}Variants"), ident.span()),
            variant_prefix: format!("Raw{ident}Variant"),
            raw_enum_fn: Ident::new(&format!("raw_{snake_case}"), ident.span()),
            discriminant_fn: Ident::new(
                &format!("raw_{snake_case}_discriminant"),
                ident.span(),
            ),
            variant_fn: Ident::new(
                &format!("raw_{snake_case}_variant"),
                ident.span(),
            ),
        }
    }

    /// Returns the identifier of the raw variant struct corresponding to the
    /// variant with the given identifier.
    pub fn variant(&self, variant: &Ident) -> Ident {
        Ident::new(
            &format!("{}{}", self.variant_prefix, variant),
            variant.span(),
        )
    }
}

/// A generated raw enum.
pub struct RawEnum {
    /// The identifiers for the generated items.
    pub idents: RawIdents,
    /// The definitions of the generated items.
    pub tokens: TokenStream,
}

impl RawEnum {
    /// Generates a raw enum for the given derive input.
    pub fn for_derive(input: &DeriveInput) -> Result<RawEnum, Error> {
        let idents = RawIdents::for_enum(&input.ident);

        let mut repr = None;
        for attr in input.attrs.iter() {
            if attr.path.is_ident("repr") {
                Repr::merge_attr(&mut repr, attr.tokens.clone())?;
            }
        }

        let data = match input.data {
            Data::Struct(ref data) => {
                return Err(Error::new_spanned(
                    data.struct_token,
                    format!("`{}` is not an enum", input.ident),
                ))
            }
            Data::Enum(ref data) => data,
            Data::Union(ref data) => {
                return Err(Error::new_spanned(
                    data.union_token,
                    format!("`{}` is not an enum", input.ident),
                ))
            }
        };

        let repr = repr.ok_or_else(|| {
            Error::new(
                Span::call_site(),
                format!(
                    "`{}` requires an explicit `repr` attribute",
                    input.ident
                ),
            )
        })?;

        let base = repr.base.ok_or_else(|| {
            Error::new(
                Span::call_site(),
                format!("`{}` requires a base `repr`", input.ident),
            )
        })?;

        let tokens = match base.kind {
            BaseKind::C => {
                derive_c(data, &input.generics, repr.primitive_type, &idents)?
            }
            BaseKind::Primitive(int) => {
                derive_primitive(data, &input.generics, int, &idents)?
            }
            BaseKind::Transparent => {
                return Err(Error::new_spanned(
                    base.kind_token,
                    format!(
                        "`{}` must have a base `repr` of `C` or a primitive",
                        input.ident,
                    ),
                ))
            }
        };

        let raw_enum_ident = &idents.r#enum;
        let discriminant_ident = &idents.discriminant;
        let raw_enum_fn_ident = &idents.raw_enum_fn;
        let discriminant_fn_ident = &idents.discriminant_fn;

        let input_ident = &input.ident;

        let (impl_generics, ty_generics, where_clause) =
            input.generics.split_for_impl();

        let tokens = quote! {
            #tokens

            fn #raw_enum_fn_ident #impl_generics (
                this: *mut #input_ident #ty_generics,
            ) -> *mut #raw_enum_ident #ty_generics
            #where_clause
            {
                #[repr(C)]
                union Reinterpret<T, U> {
                    from: ::core::mem::ManuallyDrop<T>,
                    to: ::core::mem::ManuallyDrop<U>,
                }

                let reinterpret = Reinterpret {
                    from: ::core::mem::ManuallyDrop::new(this),
                };
                // SAFETY: The input type and generated enum are guaranteed to
                // have the same layout and pointer metadata.
                ::core::mem::ManuallyDrop::into_inner(unsafe { reinterpret.to })
            }

            fn #discriminant_fn_ident #impl_generics (
                this: *mut #raw_enum_ident #ty_generics,
            ) -> *mut #discriminant_ident
            #where_clause
            {
                this.cast::<#discriminant_ident>()
            }
        };

        Ok(RawEnum { idents, tokens })
    }
}

fn generate_discriminant(
    data: &DataEnum,
    base: BaseKind,
    idents: &RawIdents,
) -> TokenStream {
    let discriminant = &idents.discriminant;

    let variants = data.variants.iter().map(|v| {
        let ident = &v.ident;
        if let Some((eq, discriminant)) = &v.discriminant {
            quote! { #ident #eq #discriminant }
        } else {
            quote! { #ident }
        }
    });

    quote! {
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        #[repr(#base)]
        #[allow(dead_code)]
        enum #discriminant {
            #(#variants),*
        }
    }
}

fn generate_variant_struct(
    variant: &Variant,
    generics: &Generics,
    idents: &RawIdents,
    with_tag: bool,
) -> TokenStream {
    let discriminant_ident = &idents.discriminant;
    let variant_struct_ident = idents.variant(&variant.ident);

    let type_params = generics.type_params().map(|p| &p.ident);
    let phantom_ty = quote! {
        ::core::marker::PhantomData<(#(#type_params,)*)>
    };
    let (impl_generics, _, where_clause) = generics.split_for_impl();

    let tag_ty = if with_tag {
        quote! { #discriminant_ident }
    } else {
        quote! { () }
    };

    match &variant.fields {
        Fields::Named(fields) => {
            let fields = fields.named.iter().map(|f| {
                let ident = &f.ident;
                let ty = &f.ty;
                quote! {
                    pub #ident: #ty
                }
            });

            quote! {
                #[repr(C)]
                #[allow(non_snake_case)]
                struct #variant_struct_ident #impl_generics #where_clause {
                    __tag: #tag_ty,
                    #(#fields,)*
                    _phantom: #phantom_ty,
                }
            }
        }
        Fields::Unnamed(fields) => {
            let fields = fields.unnamed.iter().map(|f| {
                let ty = &f.ty;
                quote! {
                    pub #ty
                }
            });

            quote! {
                #[repr(C)]
                #[allow(non_snake_case)]
                struct #variant_struct_ident #impl_generics (
                    #tag_ty,
                    #(#fields,)*
                    #phantom_ty,
                ) #where_clause;
            }
        }
        Fields::Unit => {
            quote! {
                #[repr(C)]
                #[allow(non_snake_case)]
                struct #variant_struct_ident #impl_generics (
                    #tag_ty,
                    #phantom_ty,
                ) #where_clause;
            }
        }
    }
}

fn generate_variants(
    data: &DataEnum,
    generics: &Generics,
    idents: &RawIdents,
) -> TokenStream {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let variants = data.variants.iter().map(|v| {
        let ident = &v.ident;
        let variant_struct_ident = idents.variant(ident);
        quote! { #ident (*mut #variant_struct_ident #ty_generics) }
    });

    let variants_ident = &idents.variants;

    quote! {
        #[allow(dead_code)]
        enum #variants_ident #impl_generics #where_clause {
            #(#variants,)*
        }
    }
}

fn derive_c(
    data: &DataEnum,
    generics: &Generics,
    primitive: Option<PrimitiveType>,
    idents: &RawIdents,
) -> Result<TokenStream, Error> {
    let discriminant = generate_discriminant(
        data,
        primitive
            .map(|p| BaseKind::Primitive(p.int))
            .unwrap_or(BaseKind::C),
        idents,
    );
    let variant_structs = data.variants.iter().map(|variant| {
        generate_variant_struct(variant, generics, idents, false)
    });
    let variants = generate_variants(data, generics, idents);

    let raw_ident = &idents.r#enum;
    let discriminant_ident = &idents.discriminant;
    let fields_ident = &idents.fields;
    let variants_ident = &idents.variants;
    let discriminant_fn_ident = &idents.discriminant_fn;
    let variant_fn_ident = &idents.variant_fn;

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let variant_idents = data.variants.iter().map(|v| &v.ident);
    let variant_struct_idents =
        data.variants.iter().map(|v| idents.variant(&v.ident));
    let variants_arms = data.variants.iter().map(|v| {
        let ident = &v.ident;
        let variant_struct_ident = idents.variant(&v.ident);
        quote! {
            #discriminant_ident::#ident => {
                let field = unsafe {
                    ::core::ptr::addr_of_mut!((*this).fields.#ident)
                };
                #variants_ident::#ident(
                    field.cast::<#variant_struct_ident #ty_generics>()
                )
            }
        }
    });

    Ok(quote! {
        #discriminant
        #(#variant_structs)*

        #[repr(C)]
        #[allow(non_snake_case)]
        union #fields_ident #impl_generics #where_clause {
            #(
                pub #variant_idents: ::core::mem::ManuallyDrop<
                    #variant_struct_idents #ty_generics,
                >,
            )*
        }

        #[repr(C)]
        struct #raw_ident #impl_generics #where_clause {
            pub __tag: #discriminant_ident,
            pub fields: #fields_ident #ty_generics,
        }

        #variants

        fn #variant_fn_ident #impl_generics (
            this: *mut #raw_ident #ty_generics,
        ) -> #variants_ident #ty_generics
        #where_clause
        {
            match unsafe { *#discriminant_fn_ident(this) } {
                #(#variants_arms,)*
            }
        }
    })
}

fn derive_primitive(
    data: &DataEnum,
    generics: &Generics,
    int: Int,
    idents: &RawIdents,
) -> Result<TokenStream, Error> {
    let discriminant =
        generate_discriminant(data, BaseKind::Primitive(int), idents);
    let variant_structs = data.variants.iter().map(|variant| {
        generate_variant_struct(variant, generics, idents, true)
    });
    let variants = generate_variants(data, generics, idents);

    let raw_ident = &idents.r#enum;
    let discriminant_ident = &idents.discriminant;
    let variants_ident = &idents.variants;
    let discriminant_fn_ident = &idents.discriminant_fn;
    let variant_fn_ident = &idents.variant_fn;

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let variant_idents = data.variants.iter().map(|v| &v.ident);
    let variant_struct_idents =
        data.variants.iter().map(|v| idents.variant(&v.ident));
    let variants_arms = data.variants.iter().map(|v| {
        let ident = &v.ident;
        let variant_struct_ident = idents.variant(&v.ident);
        quote! {
            #discriminant_ident::#ident => {
                let field = unsafe {
                    ::core::ptr::addr_of_mut!((*this).#ident)
                };
                #variants_ident::#ident(
                    field.cast::<#variant_struct_ident #ty_generics>()
                )
            }
        }
    });

    Ok(quote! {
        #discriminant
        #(#variant_structs)*

        #[repr(C)]
        #[allow(non_snake_case)]
        union #raw_ident #impl_generics #where_clause {
            #(
                pub #variant_idents: ::core::mem::ManuallyDrop<
                    #variant_struct_idents #ty_generics,
                >,
            )*
        }

        #variants

        fn #variant_fn_ident #impl_generics (
            this: *mut #raw_ident #ty_generics,
        ) -> #variants_ident #ty_generics
        #where_clause
        {
            match unsafe { *#discriminant_fn_ident(this) } {
                #(#variants_arms,)*
            }
        }
    })
}
