use proc_macro::TokenStream;
use syn::{Path, parse_macro_input};

mod zvalue;
use zvalue::ZValueConfig;

use crate::zvalue::{ZValueInput, impl_zvalue};

pub(crate) fn zenoh_pico_path() -> syn::Result<Path> {
    macro_utils::krate::crate_path("zenoh-pico")
}

#[proc_macro_attribute]
pub fn zvalue(args: TokenStream, input: TokenStream) -> TokenStream {
    let zvalue_config = parse_macro_input!(args as ZValueConfig);
    let zvalue_input = parse_macro_input!(input as ZValueInput);

    impl_zvalue(zvalue_input, &zvalue_config)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}
