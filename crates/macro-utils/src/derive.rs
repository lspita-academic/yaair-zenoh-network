use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

pub trait DeriveInputExtensions {
    fn impl_signature(&self, trait_tokens: Option<&TokenStream>) -> TokenStream;
}

impl DeriveInputExtensions for DeriveInput {
    fn impl_signature(&self, trait_tokens: Option<&TokenStream>) -> TokenStream {
        let ident = &self.ident;
        let (impl_genrics, type_generics, where_clause) = self.generics.split_for_impl();
        let trait_impl = trait_tokens.map(|t| quote! { #t for }).unwrap_or_default();

        quote! {
            impl #impl_genrics #trait_impl #ident #type_generics #where_clause
        }
    }
}
