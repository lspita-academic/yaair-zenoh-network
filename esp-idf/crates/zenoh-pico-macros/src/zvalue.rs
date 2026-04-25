use darling::FromMeta;
use macro_utils::derive;
use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::{DeriveInput, Fields, Ident, Path, parse::Parse, parse_quote};

use crate::{zenoh_pico_path, zenoh_pico_sys_path};

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
pub struct ZOwnConfig {
    name: String,
    #[darling(default)]
    ty: Option<Path>,
    #[darling(default)]
    zdrop: Option<ZDropAttr>,
    #[darling(default)]
    zmove: Option<ZMoveAttr>,
    #[darling(default)]
    zdefault: Option<ZDefaultAttr>,
    #[darling(default)]
    zloan: Option<ZLoanAttr>,
}

#[derive(FromMeta)]
#[darling(derive_syn_parse)]
pub struct ZViewConfig {
    name: String,
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

fn path_or_sys_default(
    value: Option<Path>,
    default_sys_ident: Ident,
    zenoh_pico_sys: &Path,
) -> syn::Result<Path> {
    syn::parse2(
        value
            .map(|ty| ty.into_token_stream())
            .unwrap_or_else(|| quote! {#zenoh_pico_sys::#default_sys_ident}),
    )
}

pub fn impl_zown(input: ZValueInput, config: ZOwnConfig) -> syn::Result<TokenStream> {
    let mut tokens = TokenStream::new();
    let mut input = input.0;

    let zenoh_pico = zenoh_pico_path()?;
    let zenoh_pico_sys = zenoh_pico_sys_path()?;
    let name = config.name;

    let zvalue_ty = path_or_sys_default(
        config.ty,
        format_ident!("z_owned_{name}_t"),
        &zenoh_pico_sys,
    )?;

    match &mut input.data {
        syn::Data::Struct(struct_data) => {
            struct_data.fields = Fields::Unnamed(syn::parse2(quote! {(#zvalue_ty)})?)
        }
        _ => panic!("Expected unit struct"),
    };

    tokens.extend(quote! {
        #[derive(Debug)]
        #input
    });

    let zvalue_trait: Path = parse_quote!(#zenoh_pico::zvalue::ZValue);
    let zvalue_impl = derive::impl_signature(&input, Some(&quote! { #zvalue_trait<#zvalue_ty> }));
    let from_impl =
        derive::impl_signature(&input, Some(&quote! { core::convert::From<#zvalue_ty> }));

    tokens.extend(quote! {
        #zvalue_impl { }

        #from_impl {
            fn from(value: #zvalue_ty) -> Self {
                Self(value)
            }
        }
    });

    let (zmove_ty_cfg, zmove_fn_cfg) = config.zmove.map(|z| (z.ty, z.zfn)).unwrap_or((None, None));
    let zmove_ty = path_or_sys_default(
        zmove_ty_cfg,
        format_ident!("z_moved_{name}_t"),
        &zenoh_pico_sys,
    )?;
    let zmove_fn = path_or_sys_default(
        zmove_fn_cfg,
        format_ident!("z_{name}_move"),
        &zenoh_pico_sys,
    )?;

    let zown_trait: Path = parse_quote!(#zenoh_pico::zvalue::ZOwn);
    let zown_impl =
        derive::impl_signature(&input, Some(&quote! { #zown_trait<#zvalue_ty, #zmove_ty> }));

    tokens.extend(quote! {
        #zown_impl {
            fn zmove(mut self) -> *mut #zmove_ty {
                unsafe { #zmove_fn(&mut self.0) }
            }
        }
    });

    let zdrop_fn = path_or_sys_default(
        config.zdrop.and_then(|z| z.zfn),
        format_ident!("z_{name}_drop"),
        &zenoh_pico_sys,
    )?;
    let drop_impl = derive::impl_signature(&input, Some(&quote! { core::ops::Drop }));
    tokens.extend(quote! {
        #drop_impl {
            fn drop(&mut self) {
                unsafe { #zdrop_fn(#zmove_fn(&mut self.0)); }
            }
        }
    });

    let zdefault_call = config.zdefault.map(|z| z.zfn).map(|zfn| {
        quote! {
            unsafe {
                #zfn(&mut zvalue);
            }
        }
    });

    let default_impl = derive::impl_signature(&input, Some(&quote! { core::default::Default }));
    tokens.extend(quote! {
        #default_impl {
            fn default() -> Self {
                let mut zvalue = Default::default();
                #zdefault_call
                Self(zvalue)
            }
        }
    });

    if let Some(zloan) = config.zloan {
        let zloan_ty = path_or_sys_default(
            zloan.ty,
            format_ident!("z_loaned_{name}_t"),
            &zenoh_pico_sys,
        )?;
        let zloan_fn =
            path_or_sys_default(zloan.zfn, format_ident!("z_{name}_loan"), &zenoh_pico_sys)?;

        let zloan_trait: Path = parse_quote!(#zenoh_pico::zvalue::ZLoan);
        let zloan_impl = derive::impl_signature(
            &input,
            Some(&quote! { #zloan_trait<#zvalue_ty, #zloan_ty> }),
        );

        tokens.extend(quote! {
            #zloan_impl {
                fn zloan(&self) -> *const #zloan_ty {
                    unsafe { #zloan_fn(&self.0) }
                }
            }
        });

        if let Some(zloan_mut) = zloan.mutable {
            let zloan_fn_mut = path_or_sys_default(
                zloan_mut.zfn,
                format_ident!("z_{name}_loan_mut"),
                &zenoh_pico_sys,
            )?;

            let zloan_mut_trait: Path = parse_quote!(#zenoh_pico::zvalue::ZLoanMut);
            let zloan_mut_impl = derive::impl_signature(
                &input,
                Some(&quote! { #zloan_mut_trait<#zvalue_ty, #zloan_ty> }),
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
