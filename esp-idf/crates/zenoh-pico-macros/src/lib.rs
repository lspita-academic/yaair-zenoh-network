use darling::FromDeriveInput;
use proc_macro::TokenStream;
use quote::ToTokens;
use syn::{DeriveInput, parse_macro_input};

mod zvalue;
use zvalue::ZValueReceiver;

pub(crate) fn zenoh_pico_path() -> syn::Result<syn::Path> {
    macro_utils::krate::crate_path("zenoh-pico")
}

#[proc_macro_derive(ZValue, attributes(zvalue))]
pub fn zvalue_derive(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    match ZValueReceiver::from_derive_input(&derive_input) {
        Ok(receiver) => receiver.into_token_stream().into(),
        Err(e) => e.write_errors().into(),
    }
}
