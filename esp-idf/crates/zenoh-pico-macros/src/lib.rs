use darling::FromDeriveInput;
use proc_macro::TokenStream;
use syn::{DeriveInput, Path, parse_macro_input};

mod zvalue;
use zvalue::ZValueConfig;

use crate::zvalue::{ZValueInput, impl_zvalue};

pub(crate) fn zenoh_pico_path() -> syn::Result<Path> {
    macro_utils::krate::crate_path("zenoh-pico")
}

pub(crate) fn zenoh_pico_sys_path() -> syn::Result<Path> {
    macro_utils::krate::crate_path("zenoh-pico-sys")
}

#[proc_macro_attribute]
pub fn zvalue(args: TokenStream, input: TokenStream) -> TokenStream {
    let zvalue_config = parse_macro_input!(args as ZValueConfig);
    let derive_input = parse_macro_input!(input as DeriveInput);
    let zvalue_input = match ZValueInput::from_derive_input(&derive_input) {
        Ok(z) => z,
        Err(e) => return e.write_errors().into(),
    };

    impl_zvalue(zvalue_input, &zvalue_config)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}
