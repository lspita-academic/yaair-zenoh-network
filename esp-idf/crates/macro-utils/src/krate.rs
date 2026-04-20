use proc_macro_crate::FoundCrate;
use proc_macro2::Span;
use syn::{Error, Ident, Path};

pub fn crate_path(name: &str) -> syn::Result<Path> {
    let span = Span::call_site();
    let found_crate = proc_macro_crate::crate_name(name)
        .map_err(|e| Error::new(span, format!("Error searching crate {name}: {e}")))?;

    let crate_name = match found_crate {
        FoundCrate::Itself => String::from("crate"),
        FoundCrate::Name(name) => name,
    };
    Ok(Path::from(Ident::new(&crate_name, span)))
}
