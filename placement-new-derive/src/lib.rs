//! "placement-new" derive macros

#![forbid(unsafe_code)]
#![deny(missing_docs, clippy::all, clippy::cargo)]

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::parse::Parse;
use syn::{Data, DataStruct, DeriveInput, Field, Fields, Ident, ItemStruct, Path, Token, Type};

macro_rules! emit_error {
    ($token:expr, $msg: expr) => {{
        return TokenStream::from(syn::Error::new_spanned($token, $msg).to_compile_error());
    }};
    (@repr_error $token:expr) => {{
        emit_error!(
            $token,
            "Only repr(C) structs and enums can derive UninitProject"
        )
    }};
}

/// Implements `UninitProject` for a type and generates corresponding types.
#[proc_macro_derive(UninitProject)]
pub fn uninit_project(input: TokenStream) -> TokenStream {
    impl_UninitProject(&syn::parse(input).unwrap())
}

#[allow(non_snake_case)]
fn impl_UninitProject(ast: &DeriveInput) -> TokenStream {
    let has_repr_c = ast.attrs.iter().any(|attr| match attr.path.get_ident() {
        Some(ident) => *ident == "repr" && attr.tokens.to_string() == "(C)",
        None => false,
    });

    if !has_repr_c {
        emit_error!(@repr_error ast)
    }

    match ast.data {
        Data::Struct(_) => impl_UninitProject_for_struct(ast),
        Data::Enum(_) => impl_UninitProject_for_enum(ast),
        Data::Union(_) => emit_error!(@repr_error ast),
    }
}

fn clone_and_modify<T: Clone>(origin: &T, f: impl FnOnce(&mut T)) -> T {
    let mut cloned = origin.clone();
    f(&mut cloned);
    cloned
}

fn project_fields(fields: &Fields) -> Fields {
    let project_ty = |field: &mut Field| {
        let ty = &field.ty;
        field.ty =
            Type::Verbatim(quote! { ::placement_new::__private::core::mem::MaybeUninit<#ty> });
    };

    clone_and_modify(fields, |new_fields| match new_fields {
        Fields::Named(ref mut fields) => fields.named.iter_mut().for_each(project_ty),
        Fields::Unnamed(ref mut fields) => fields.unnamed.iter_mut().for_each(project_ty),
        Fields::Unit => {}
    })
}

#[allow(non_snake_case)]
fn impl_UninitProject_for_struct(ast: &DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let uninit_name = format_ident!("__UninitStruct__{}", name);

    let data = match ast.data {
        Data::Struct(ref data) => data,
        _ => unreachable!(),
    };

    if let Fields::Unit = data.fields {
        emit_error!(ast, "unexpected unit struct")
    }

    let projected_struct = clone_and_modify(ast, |new_ast| {
        new_ast.ident = uninit_name.clone();
        match new_ast.data {
            Data::Struct(DataStruct { ref mut fields, .. }) => *fields = project_fields(fields),
            _ => unreachable!(),
        }
    });

    let codegen = quote! {
        #[doc(hidden)]
        #projected_struct

        #[doc(hidden)]
        unsafe impl ::placement_new::UninitProject<#uninit_name> for #name {
            fn uninit_project(this: &mut MaybeUninit<Self>) -> &mut #uninit_name {
                unsafe { &mut *this.as_mut_ptr().cast() }
            }
        }
    };

    codegen.into()
}

#[allow(non_snake_case)]
fn impl_UninitProject_for_enum(ast: &DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let discriminant_name = format_ident!("__UninitEnumDiscriminant__{}", name);

    let data = match ast.data {
        Data::Enum(ref data) => data,
        _ => unreachable!(),
    };

    let discriminant_enum = clone_and_modify(ast, |new_ast| {
        new_ast.ident = discriminant_name.clone();
        let new_data = match new_ast.data {
            Data::Enum(ref mut data) => data,
            _ => unreachable!(),
        };
        new_data
            .variants
            .iter_mut()
            .for_each(|variant| variant.fields = Fields::Unit)
    });

    let mut variant_structs: Vec<_> = Vec::new();
    for v in &data.variants {
        variant_structs.push(ItemStruct {
            attrs: v.attrs.clone(),
            vis: ast.vis.clone(),
            generics: ast.generics.clone(),
            ident: format_ident!("__UninitEnumVariant__{}__{}", name, v.ident),
            struct_token: Default::default(),
            fields: project_fields(&v.fields),
            semi_token: match v.fields {
                Fields::Named(_) => None,
                Fields::Unnamed(_) => Some(Default::default()),
                Fields::Unit => Some(Default::default()),
            },
        });
    }

    let mut impls: Vec<_> = Vec::new();
    let mut fns: Vec<_> = Vec::new();
    for (v, vs) in data.variants.iter().zip(variant_structs.iter()) {
        let variant_name = &v.ident;
        let uninit_name = &vs.ident;
        impls.push(quote! {
            #[doc(hidden)]
            unsafe impl ::placement_new::UninitProject<#uninit_name> for #name {
                fn uninit_project(this: &mut ::placement_new::__private::core::mem::MaybeUninit<Self>) -> &mut #uninit_name {
                    type Tag = #discriminant_name;
                    type Payload = #uninit_name;
                    unsafe{
                        let base = this.as_mut_ptr().cast();
                        let (tag, payload) = ::placement_new::__private::split_enum::<Tag, Payload>(base);
                        tag.write(Tag::#variant_name);
                        &mut *payload
                    }
                }
            }
        });

        let vis = &ast.vis;
        let fn_name = format_ident!("__uninit_project_variant__{}", variant_name);

        fns.push(quote! {
            #[allow(non_snake_case)]
            #vis fn #fn_name (this: &mut ::placement_new::__private::core::mem::MaybeUninit<Self>) -> &mut #uninit_name {
                Self::uninit_project(this)
            }
        })
    }

    let codegen = quote! {
        #[doc(hidden)]
        #discriminant_enum

        #(
            #[repr(C)]
            #[doc(hidden)]
            #variant_structs
        )*

        #(#impls)*

        #[doc(hidden)]
        impl #name {
            #(#fns)*
        }
    };

    codegen.into()
}

#[doc(hidden)]
#[proc_macro]
pub fn __uninit_project_variant(tt: TokenStream) -> TokenStream {
    struct ProjectorPath {
        ty_path: Path,
        variant: Ident,
    }

    impl Parse for ProjectorPath {
        fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
            let ty_path = input.parse::<Path>()?;
            let _ = input.parse::<Token![=>]>()?;
            let variant = input.parse::<Ident>()?;
            Ok(Self { ty_path, variant })
        }
    }

    let ProjectorPath { ty_path, variant } = syn::parse_macro_input!(tt as ProjectorPath);
    let fn_name = format_ident!("__uninit_project_variant__{}", variant);
    let codegen = quote! { #ty_path :: #fn_name };
    codegen.into()
}
