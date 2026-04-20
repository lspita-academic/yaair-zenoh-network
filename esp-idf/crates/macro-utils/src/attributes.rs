use itertools::Itertools;
use proc_macro2::Span;
use syn::{Attribute, Error, parse::Parse};

pub fn parse_single_attribute<T: Parse>(attrs: &[Attribute], name: &str) -> syn::Result<Option<T>> {
    let Some(attr) = attrs
        .iter()
        .filter(|a| a.path().is_ident(name))
        .at_most_one()
        .map_err(|_| Error::new(Span::call_site(), format!("Duplicate attribute {name}")))?
    else {
        return Ok(None);
    };

    let args = attr.parse_args::<T>()?;
    Ok(Some(args))
}
