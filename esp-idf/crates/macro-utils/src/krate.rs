use proc_macro_crate::FoundCrate;
use proc_macro2::Span;
use syn::{Error, Ident, Path, parse_quote};

pub fn crate_path(name: &str) -> syn::Result<Path> {
    let span = Span::call_site();
    let found_crate = proc_macro_crate::crate_name(name)
        .map_err(|e| Error::new(span, format!("Error searching crate {name}: {e}")))?;

    let crate_path = match found_crate {
        FoundCrate::Itself => parse_quote!(crate),
        FoundCrate::Name(name) => {
            let ident = Ident::new(&name, span);
            parse_quote!(::#ident)
        },
    };
    Ok(crate_path)
}
