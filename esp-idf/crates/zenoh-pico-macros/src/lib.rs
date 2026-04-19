use proc_macro::TokenStream;

mod zvalue;

#[proc_macro_derive(ZValue)]
pub fn derive_z_value(input: TokenStream) -> TokenStream {
    zvalue::derive_macro(input)
}
