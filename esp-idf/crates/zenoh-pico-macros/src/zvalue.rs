use itertools::Itertools;
use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Index, Member, parse_macro_input};

pub fn derive_macro(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_ident = &input.ident;
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();

    let fields = match &input.data {
        Data::Struct(s) => &s.fields,
        _ => panic!("ZValue can only be derived for structs"),
    };

    let zvalue_field = fields
        .iter()
        .exactly_one()
        .unwrap_or_else(|_| panic!("ZValue can be derived only for structs with one field")); // expect required debug trait
    let zvalue_member = zvalue_field
        .ident
        .clone()
        .map(|i| Member::Named(i))
        .unwrap_or(Member::Unnamed(Index::from(0))); // fields of tuple structs have no names.
    let zvalue_type = zvalue_field.ty.clone();

    let init_from_value = match &zvalue_member {
        Member::Named(ident) => quote! {
            Self { #ident = value }
        },
        Member::Unnamed(_) => quote! {
            Self(value)
        },
    };

    quote! {
        impl #impl_generics ZValue<#zvalue_type> for #struct_ident #type_generics #where_clause {
            fn zvalue(&self) -> &#zvalue_type {
                &self.#zvalue_member
            }

            fn zvalue_mut(&mut self) -> &mut #zvalue_type {
                &mut self.#zvalue_member
            }
        }

        impl #impl_generics Into<#zvalue_type> for #struct_ident #type_generics #where_clause {
            fn into(self) -> #zvalue_type {
                self.#zvalue_member
            }
        }

        impl #impl_generics From<#zvalue_type> for #struct_ident #type_generics #where_clause {
            fn from(value: #zvalue_type) -> Self {
                #init_from_value
            }
        }
    }
    .into()
}
