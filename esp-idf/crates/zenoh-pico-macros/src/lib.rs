mod zvalue;

use proc_macro::TokenStream;
use quote::quote;
use syn::{Path, parse_macro_input};

use crate::zvalue::{ZClosureConfig, ZOwnConfig, ZValueInput, ZWrapConfig};

pub(crate) fn zenoh_pico_path() -> syn::Result<Path> {
    macro_utils::krate::crate_path("zenoh-pico")
}

pub(crate) fn zenoh_pico_sys_path() -> syn::Result<Path> {
    let zenoh_pico = zenoh_pico_path()?;
    syn::parse2(quote! {#zenoh_pico::sys})
}

#[proc_macro_attribute]
pub fn zwrap(args: TokenStream, input: TokenStream) -> TokenStream {
    let zwrap_config = parse_macro_input!(args as ZWrapConfig);
    let zvalue_input = parse_macro_input!(input as ZValueInput);

    zvalue::zwrap(zvalue_input, zwrap_config)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

#[proc_macro_attribute]
pub fn zown(args: TokenStream, input: TokenStream) -> TokenStream {
    let zown_config = parse_macro_input!(args as ZOwnConfig);
    let zvalue_input = parse_macro_input!(input as ZValueInput);

    zvalue::zown(zvalue_input, zown_config)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

#[proc_macro_attribute]
pub fn zclosure(args: TokenStream, input: TokenStream) -> TokenStream {
    let zclosure_config = parse_macro_input!(args as ZClosureConfig);
    let zvalue_input = parse_macro_input!(input as ZValueInput);

    zvalue::zclosure(zvalue_input, zclosure_config)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}
