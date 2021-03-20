//! "placement-new" derive macros

#![forbid(unsafe_code)]
#![deny(missing_docs, clippy::all, clippy::cargo)]

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::spanned::Spanned;
use syn::{DataStruct, DeriveInput, FieldsNamed, Type};

/// Implements `UninitProject` for a type and generates corresponding types.
#[proc_macro_derive(UninitProject)]
pub fn uninit_project_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_uninit_project(&ast)
}

macro_rules! emit_error {
    ($token:expr, $msg: expr) => {{
        return TokenStream::from(syn::Error::new($token.span(), $msg).to_compile_error());
    }};
}

fn impl_uninit_project(ast: &DeriveInput) -> TokenStream {
    let has_repr_c = ast.attrs.iter().any(|attr| match attr.path.get_ident() {
        Some(ident) => *ident == "repr" && attr.tokens.to_string() == "(C)",
        None => false,
    });

    if !has_repr_c {
        emit_error!(ast, "Only repr(C) structs can derive UninitProject now")
    }

    let name = &ast.ident;
    let uninit_name = format_ident!("__Uninit{}", name);

    let fields = match ast.data {
        syn::Data::Struct(DataStruct { ref fields, .. }) => match fields {
            syn::Fields::Named(fields) => fields,
            syn::Fields::Unnamed(_) => emit_error!(ast, "unexpected tuple struct"),
            syn::Fields::Unit => emit_error!(ast, "unexpected unit struct"),
        },
        _ => emit_error!(ast, "expected a struct"),
    };

    let projected_fields = FieldsNamed {
        brace_token: fields.brace_token,
        named: fields
            .named
            .iter()
            .map(|field| {
                let original_ty = &field.ty;
                let projected_ty =
                    Type::Verbatim(quote! { ::core::mem::MaybeUninit<#original_ty> });
                let mut projected_field = field.clone();
                projected_field.ty = projected_ty;
                projected_field
            })
            .collect(),
    };

    let projected_struct = {
        let mut projected_ast = ast.clone();
        projected_ast.ident = uninit_name.clone();
        match projected_ast.data {
            syn::Data::Struct(DataStruct {
                fields: syn::Fields::Named(ref mut fields),
                ..
            }) => *fields = projected_fields,
            _ => unreachable!(),
        }
        projected_ast
    };

    let gen = quote! {
        #[doc(hidden)]
        #projected_struct

        #[doc(hidden)]
        unsafe impl ::placement_new::UninitProject for #name {
            type Output = #uninit_name;
        }
    };

    gen.into()
}
