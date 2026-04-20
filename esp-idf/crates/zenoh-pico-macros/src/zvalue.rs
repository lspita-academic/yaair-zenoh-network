use itertools::Itertools;
use macro_utils::{attributes, derive};
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    Data, DeriveInput, Fields, Path, Token, Type,
    parse::{Parse, ParseStream},
    parse_quote,
};

use crate::zenoh_pico_path;

struct ZDropAttribute {
    callable: Path,
}

impl Parse for ZDropAttribute {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let callable: Path = input.parse()?;

        if input.is_empty() {
            Ok(Self { callable })
        } else {
            Err(input.error("Expected arguments: fn"))
        }
    }
}

struct ZMoveAttribute {
    ty: Type,
    callable: Path,
}

impl Parse for ZMoveAttribute {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ty: Type = input.parse()?;
        let _comma: Token![,] = input.parse()?;
        let callable: Path = input.parse()?;

        if input.is_empty() {
            Ok(Self { ty, callable })
        } else {
            Err(input.error("Expected arguments: Type, fn"))
        }
    }
}

struct ZDefaultAttribute {
    callable: Path,
}

impl Parse for ZDefaultAttribute {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let callable: Path = input.parse()?;

        if input.is_empty() {
            Ok(Self { callable })
        } else {
            Err(input.error("Expected arguments: fn"))
        }
    }
}

struct ZLoanAttribute {
    ty: Type,
    callable: Path,
    callable_mut: Option<Path>,
}

impl Parse for ZLoanAttribute {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ty: Type = input.parse()?;
        let _comma: Token![,] = input.parse()?;
        let callable: Path = input.parse()?;
        let mut callable_mut = None::<Path>;

        // Reject trailing tokens: #[zmove(Foo, bar, extra)] is an error
        if !input.is_empty() {
            let _comma: Token![,] = input.parse()?;
            callable_mut = Some(input.parse()?);
        }

        if input.is_empty() {
            Ok(Self {
                ty,
                callable,
                callable_mut,
            })
        } else {
            Err(input.error("Expected arguments: Type, fn, [fn_mut]"))
        }
    }
}

pub struct ZValueDerive {
    derive_input: DeriveInput,
    zvalue_type: Type,
    zdrop: ZDropAttribute,
    zmove: ZMoveAttribute,
    zdefault: Option<ZDefaultAttribute>,
    zloan: Option<ZLoanAttribute>,
}

impl Parse for ZValueDerive {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let derive_input = input.parse::<DeriveInput>()?;
        let struct_data = match derive_input.data.clone() {
            Data::Struct(s) => Ok(s),
            _ => Err(input.error("Zenoh type wrapper must be a struct")),
        }?;
        let fields = match &struct_data.fields {
            Fields::Unnamed(fields_unnamed) => fields_unnamed,
            _ => return Err(input.error("Zenoh type wrapper must be a tuple struct")),
        };
        let zvalue_type = fields
            .unnamed
            .iter()
            .exactly_one()
            .map_err(|_| input.error("Zenoh type wrapper must have exactly one field"))?
            .ty
            .clone();

        let attrs = &derive_input.attrs;
        let Some(zdrop) = attributes::parse_single_attribute(attrs, "zdrop")? else {
            return Err(input.error("Missing required attribute zdrop"));
        };
        let Some(zmove) = attributes::parse_single_attribute(attrs, "zmove")? else {
            return Err(input.error("Missing required attribute zmove"));
        };
        let zdefault = attributes::parse_single_attribute(attrs, "zdefault")?;
        let zloan = attributes::parse_single_attribute(attrs, "zloan")?;

        Ok(Self {
            derive_input,
            zvalue_type,
            zdrop,
            zmove,
            zloan,
            zdefault,
        })
    }
}

impl ToTokens for ZValueDerive {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let zenoh_pico_path = zenoh_pico_path().unwrap();

        let zvalue_trait_path: Path = parse_quote!(#zenoh_pico_path::zvalue::ZValue);
        let zvalue_type = &self.zvalue_type;
        let zmove_type = &self.zmove.ty;
        let zmove_callable = &self.zmove.callable;
        let zdrop_callable = &self.zdrop.callable;

        let zvalue_impl = derive::impl_signature(
            &self.derive_input,
            Some(&quote! { #zvalue_trait_path<#zvalue_type, #zmove_type> }),
        );
        let zvalue_from_impl = derive::impl_signature(
            &self.derive_input,
            Some(&quote! { core::convert::From<#zvalue_type> }),
        );
        let zvalue_drop_impl =
            derive::impl_signature(&self.derive_input, Some(&quote! { core::ops::Drop }));
        let zvalue_default_impl =
            derive::impl_signature(&self.derive_input, Some(&quote! { core::default::Default }));

        tokens.extend(quote! {
            #zvalue_impl {
                fn zmove(mut self) -> *mut #zmove_type {
                    unsafe { #zmove_callable(&mut self.0) }
                }
            }

            #zvalue_from_impl {
                fn from(value: #zvalue_type) -> Self {
                    Self(value)
                }
            }

            #zvalue_drop_impl {
                fn drop(&mut self) {
                    // using zmove() directly requires ownership of self
                    unsafe { #zdrop_callable(#zmove_callable(&mut self.0)) };
                }
            }
        });

        let zdefault_extra_impl = if let Some(zdefault) = &self.zdefault {
            let zdefault_callable = &zdefault.callable;

            quote! {
                unsafe {
                    #zdefault_callable(&mut zvalue);
                }
            }
        } else {
            Default::default()
        };

        tokens.extend(quote! {
            #zvalue_default_impl {
                fn default() -> Self {
                    let mut zvalue = Default::default();
                    #zdefault_extra_impl
                    Self(zvalue)
                }
            }
        });

        if let Some(zloan) = &self.zloan {
            let zloan_trait_path: Path = parse_quote!(#zenoh_pico_path::zvalue::ZLoan);
            let zloan_type = &zloan.ty;
            let zloan_callable = &zloan.callable;

            let zloan_impl = derive::impl_signature(
                &self.derive_input,
                Some(&quote! { #zloan_trait_path<#zvalue_type, #zmove_type, #zloan_type> }),
            );

            tokens.extend(quote! {
                #zloan_impl {
                    fn zloan(&self) -> *const #zloan_type {
                        unsafe { #zloan_callable(&self.0) }
                    }
                }
            });

            if let Some(zloan_callable_mut) = &zloan.callable_mut {
                let zloan_trait_path_mut: Path = parse_quote!(#zenoh_pico_path::zvalue::ZLoanMut);

                let zloan_mut_impl = derive::impl_signature(
                    &self.derive_input,
                    Some(&quote! { #zloan_trait_path_mut<#zvalue_type, #zmove_type, #zloan_type> }),
                );

                tokens.extend(quote! {
                    #zloan_mut_impl {
                        fn zloan_mut(&mut self) -> *mut #zloan_type {
                            unsafe { #zloan_callable_mut(&mut self.0) }
                        }
                    }
                });
            }
        }
    }
}
