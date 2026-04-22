use darling::FromMeta;
use macro_utils::derive;
use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::{DeriveInput, Fields, Path, parse::Parse, parse_quote};

use crate::zenoh_pico_path;

#[derive(FromMeta, Default, Clone)]
#[darling(default)]
struct ZOwnAttr {
    ty: Option<Path>,
}

#[derive(FromMeta, Default, Clone)]
#[darling(default)]
struct ZDropAttr {
    zfn: Option<Path>,
}

#[derive(FromMeta, Default, Clone)]
#[darling(default)]
struct ZMoveAttr {
    ty: Option<Path>,
    zfn: Option<Path>,
}

#[derive(FromMeta, Clone)]
struct ZDefaultAttr {
    zfn: Path,
}

#[derive(FromMeta, Default, Clone)]
#[darling(default, from_word = || FromMeta::from_list(&[]))]
struct ZLoanMutAttr {
    zfn: Option<Path>,
}

#[derive(FromMeta, Default, Clone)]
#[darling(default, from_word = || FromMeta::from_list(&[]))]
struct ZLoanAttr {
    ty: Option<Path>,
    zfn: Option<Path>,
    mutable: Option<ZLoanMutAttr>,
}

#[derive(FromMeta)]
#[darling(derive_syn_parse)]
pub struct ZValueConfig {
    name: String,
    #[darling(default)]
    zown: Option<ZOwnAttr>,
    #[darling(default)]
    zdrop: Option<ZDropAttr>,
    #[darling(default)]
    zmove: Option<ZMoveAttr>,
    #[darling(default)]
    zdefault: Option<ZDefaultAttr>,
    #[darling(default)]
    zloan: Option<ZLoanAttr>,
}

pub struct ZValueInput(DeriveInput);

impl Parse for ZValueInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let derive_input = input.parse::<DeriveInput>()?;

        match &derive_input.data {
            syn::Data::Struct(s) if matches!(s.fields, Fields::Unit) => {}
            syn::Data::Struct(_) => return Err(input.error("Struct must be a unit struct")),
            _ => return Err(input.error("Only unit structs are supported")),
        };

        Ok(Self(derive_input))
    }
}

pub fn impl_zvalue(input: ZValueInput, config: &ZValueConfig) -> syn::Result<TokenStream> {
    let mut tokens = TokenStream::new();
    let mut input = input.0;

    let zenoh_pico = zenoh_pico_path()?;
    let zenoh_pico_sys = quote! {#zenoh_pico::sys};
    let name = &config.name;

    let zown_ty = &config
        .zown
        .clone()
        .map(|z| z.ty.into_token_stream())
        .unwrap_or_else(|| {
            let owned_t = format_ident!("z_owned_{name}_t");
            quote! { #zenoh_pico_sys::#owned_t }
        });

    match &mut input.data {
        syn::Data::Struct(struct_data) => {
            struct_data.fields = Fields::Unnamed(parse_quote!((#zown_ty)))
        }
        _ => panic!("Expected unit struct"),
    };

    tokens.extend(quote! {
        #[derive(Debug)]
        #input
    });

    let zdrop_fn = &config
        .zdrop
        .clone()
        .map(|z| z.zfn.into_token_stream())
        .unwrap_or_else(|| {
            let zdrop_fn = format_ident!("z_{name}_drop");
            quote! { #zenoh_pico_sys::#zdrop_fn }
        });
    let zmove_ty = &config
        .zmove
        .clone()
        .map(|z| z.ty.into_token_stream())
        .unwrap_or_else(|| {
            let moved_t = format_ident!("z_moved_{name}_t");
            quote! { #zenoh_pico_sys::#moved_t }
        });
    let zmove_fn = &config
        .zmove
        .clone()
        .map(|z| z.zfn.into_token_stream())
        .unwrap_or_else(|| {
            let zmove_t = format_ident!("z_{name}_move");
            quote! { #zenoh_pico_sys::#zmove_t }
        });

    let zvalue_trait: Path = parse_quote!(#zenoh_pico::zvalue::ZValue);
    let zvalue_impl =
        derive::impl_signature(&input, Some(&quote! { #zvalue_trait<#zown_ty, #zmove_ty> }));
    let from_impl = derive::impl_signature(&input, Some(&quote! { core::convert::From<#zown_ty> }));
    let drop_impl = derive::impl_signature(&input, Some(&quote! { core::ops::Drop }));

    tokens.extend(quote! {
        #zvalue_impl {
            fn zmove(mut self) -> *mut #zmove_ty {
                unsafe { #zmove_fn(&mut self.0) }
            }
        }

        #from_impl {
            fn from(value: #zown_ty) -> Self {
                Self(value)
            }
        }

        #drop_impl {
            fn drop(&mut self) {
                unsafe { #zdrop_fn(#zmove_fn(&mut self.0)) };
            }
        }
    });

    let zdefault_init = &config
        .zdefault
        .clone()
        .map(|z| {
            let zdefault_fn = &z.zfn;
            quote! { unsafe { #zdefault_fn(&mut zvalue); } }
        })
        .unwrap_or_default();

    let default_impl = derive::impl_signature(&input, Some(&quote! { core::default::Default }));
    tokens.extend(quote! {
        #default_impl {
            fn default() -> Self {
                let mut zvalue = Default::default();
                #zdefault_init
                Self(zvalue)
            }
        }
    });

    if let Some(zloan) = &config.zloan {
        let zloan_trait: Path = parse_quote!(#zenoh_pico::zvalue::ZLoan);
        let zloan_ty = &zloan
            .ty
            .clone()
            .map(|ty| ty.into_token_stream())
            .unwrap_or_else(|| {
                let loaned_t = format_ident!("z_loaned_{name}_t");
                quote! { #zenoh_pico_sys::#loaned_t }
            });
        let zloan_fn = &zloan
            .zfn
            .clone()
            .map(|zfn| zfn.into_token_stream())
            .unwrap_or_else(|| {
                let zloan_fn = format_ident!("z_{name}_loan");
                quote! { #zenoh_pico_sys::#zloan_fn }
            });

        let zloan_impl = derive::impl_signature(
            &input,
            Some(&quote! { #zloan_trait<#zown_ty, #zmove_ty, #zloan_ty> }),
        );

        tokens.extend(quote! {
            #zloan_impl {
                fn zloan(&self) -> *const #zloan_ty {
                    unsafe { #zloan_fn(&self.0) }
                }
            }
        });

        if let Some(zloan_mut) = &zloan.mutable {
            let zloan_mut_trait: Path = parse_quote!(#zenoh_pico::zvalue::ZLoanMut);
            let zloan_fn_mut = &zloan_mut
                .zfn
                .clone()
                .map(|zfn| zfn.into_token_stream())
                .unwrap_or_else(|| {
                    let zloan_fn_mut = format_ident!("z_{name}_loan_mut");
                    quote! { #zenoh_pico_sys::#zloan_fn_mut }
                });

            let zloan_mut_impl = derive::impl_signature(
                &input,
                Some(&quote! { #zloan_mut_trait<#zown_ty, #zmove_ty, #zloan_ty> }),
            );

            tokens.extend(quote! {
                #zloan_mut_impl {
                    fn zloan_mut(&mut self) -> *mut #zloan_ty {
                        unsafe { #zloan_fn_mut(&mut self.0) }
                    }
                }
            });
        }
    }

    Ok(tokens)
}
