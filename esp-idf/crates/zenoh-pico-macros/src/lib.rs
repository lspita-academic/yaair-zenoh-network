use proc_macro::TokenStream;
use quote::ToTokens;
use syn::{Path, parse_macro_input};

mod zvalue;

use zvalue::ZValueDerive;

pub(crate) fn zenoh_pico_path() -> syn::Result<Path> {
    macro_utils::krate::crate_path("zenoh-pico")
}

#[proc_macro_derive(ZValue, attributes(zdrop, zmove, zloan, zdefault))]
pub fn zvalue_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ZValueDerive);
    input.into_token_stream().into()
}
