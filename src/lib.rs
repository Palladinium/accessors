//! Support for `#[derive(accesors)]`.  Based on the [example code][] for
//! syn.
//!
//! [example code]: https://github.com/dtolnay/syn

// I threw this code together in just a few minutes, and it could use a
// good refactoring once I figure out the basic ideas.  Do not use use this
// as an example of good style.

extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate quote;
extern crate syn;

use proc_macro::TokenStream;
use proc_macro2::Span;
use std::collections::BTreeMap;
use syn::{Field, Lit, LitBool, Meta, NestedMeta};

#[proc_macro_derive(getters, attributes(getter))]
pub fn derive_getters(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();

    let fields: Vec<_> = match ast.data {
        syn::Data::Struct(ref s) => s
            .fields
            .iter()
            .map(|f| (f.ident.as_ref().unwrap(), &f.ty, GetterConfig::new(&f)))
            .collect(),
        _ => panic!("#[derive(getters)] can only be used with braced structs"),
    };

    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) =
        ast.generics.split_for_impl();

    let getters: Vec<_> = fields
        .iter()
        .map(|&(ref field_name, ref ty, ref _config)| {
            let get_fn_name = field_name;

            quote! {
                pub fn #get_fn_name(&self) -> &#ty {
                    &self.#field_name
                }
            }
        })
        .collect();

    let expanded = quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            #(#getters)*
        }
    };

    expanded.into()
}

#[proc_macro_derive(setters, attributes(setter))]
pub fn derive_setters(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();

    let fields: Vec<_> = match ast.data {
        syn::Data::Struct(ref s) => s
            .fields
            .iter()
            .map(|f| (f.ident.as_ref().unwrap(), &f.ty, SetterConfig::new(&f)))
            .collect(),
        _ => panic!("#[derive(setters)] can only be used with braced structs"),
    };

    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) =
        ast.generics.split_for_impl();
    let setters: Vec<_> = fields
        .iter()
        .map(|&(ref field_name, ref ty, ref config)| {
            let set_fn_name = syn::Ident::new(
                &format!("set_{}", field_name),
                Span::call_site(),
            );
            if config.into {
                quote! {
                    pub fn #set_fn_name<T>(&mut self, value: T)
                        where T: Into<#ty>
                    {
                        self.#field_name = value.into();
                    }
                }
            } else {
                quote! {
                    pub fn #set_fn_name(&mut self, value: #ty) {
                        self.#field_name = value;
                    }
                }
            }
        })
        .collect();

    let expanded = quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            #(#setters)*
        }
    };

    expanded.into()
}

struct SetterConfig {
    into: bool,
}

impl SetterConfig {
    fn new(field: &Field) -> Self {
        let config = extract_attr(field, "setter");

        let into = config.get("into").map(extract_bool).unwrap_or(false);

        Self { into }
    }
}

struct GetterConfig {}

impl GetterConfig {
    fn new(_field: &Field) -> Self {
        Self {}
    }
}

fn extract_bool(lit: &Lit) -> bool {
    if let Lit::Bool(LitBool { value, .. }) = lit {
        value.clone()
    } else {
        panic!("Expected bool");
    }
}

fn extract_attr(field: &Field, name: &str) -> BTreeMap<String, Lit> {
    let matching_meta_lists: Vec<_> = field
        .attrs
        .iter()
        .filter(|a| match a.style {
            syn::AttrStyle::Outer => true,
            _ => false,
        })
        .filter_map(|a| a.interpret_meta())
        .filter(|m| m.name() == name)
        .filter_map(|m| match m {
            Meta::List(l) => Some(l),
            _ => None,
        })
        .collect();

    match matching_meta_lists.len() {
        0 => BTreeMap::new(),
        1 => matching_meta_lists[0]
            .nested
            .iter()
            .map(|e| match e {
                NestedMeta::Meta(Meta::Word(ref ident)) => (
                    ident.to_string(),
                    Lit::Bool(LitBool {
                        value: true,
                        span: Span::call_site(),
                    }),
                ),
                NestedMeta::Meta(Meta::NameValue(ref kv)) => {
                    (kv.ident.to_string(), kv.lit.clone())
                }

                _ => panic!("Malformed {} attribute", name),
            })
            .collect(),
        _ => panic!("Expected at most one {} attribute", name),
    }
}
