use std::ops::Deref;

use darling::FromMeta;
use macro_utils::derive;
use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::{DeriveInput, Fields, Ident, Path, parse::Parse, parse_quote};

use crate::{zenoh_pico_path, zenoh_pico_sys_path};

#[derive(FromMeta, Default, Clone)]
#[darling(default)]
struct ZValueAttr {
    ty: Option<Path>,
}

#[derive(FromMeta, Default, Clone)]
#[darling(default)]
struct ZMoveAttr {
    ty: Option<Path>,
    zfn: Option<Path>,
    drop_zfn: Option<Path>,
}

#[derive(FromMeta, Default, Clone)]
#[darling(default)]
struct ZDefaultAttr {
    zfn: Option<Path>,
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
    base: String,
    #[darling(default)]
    zvalue: Option<ZValueAttr>,
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
    base: String,
    #[darling(default)]
    zvalue: Option<ZValueAttr>,
    #[darling(default)]
    zdefault: Option<ZDefaultAttr>,
    #[darling(default)]
    zloan: Option<ZLoanAttr>,
}

pub struct ZValueInput(DeriveInput);

impl Deref for ZValueInput {
    type Target = DeriveInput;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

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

impl ToTokens for ZValueInput {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens);
    }
}

impl ZValueInput {
    fn transform_struct(&mut self, zvalue_ty: &Path) -> syn::Result<TokenStream> {
        match &mut self.0.data {
            syn::Data::Struct(struct_data) => {
                struct_data.fields = Fields::Unnamed(syn::parse2(quote! {(#zvalue_ty)})?)
            }
            _ => panic!("Expected unit struct"),
        };

        Ok(quote! {
            #[derive(Debug)]
            #self
        })
    }
}

fn path_or_sys_default(
    value: Option<&Path>,
    default_sys_ident: &Ident,
    zenoh_pico_sys: &Path,
) -> syn::Result<Path> {
    syn::parse2(
        value
            .map(|ty| ty.into_token_stream())
            .unwrap_or_else(|| quote! {#zenoh_pico_sys::#default_sys_ident}),
    )
}

fn zvalue_type_path(
    zvalue: Option<&ZValueAttr>,
    base: &str,
    zenoh_pico_sys: &Path,
) -> syn::Result<Path> {
    path_or_sys_default(
        zvalue.and_then(|z| z.ty.as_ref()),
        &format_ident!("z_owned_{base}_t"),
        zenoh_pico_sys,
    )
}

struct AttrParams<'a> {
    base: &'a String,
    input: &'a ZValueInput,
    zenoh_pico: &'a Path,
    zenoh_pico_sys: &'a Path,
    zvalue_ty: &'a Path,
}

trait AttrToTokens {
    fn attr_tokens(&self, attr_params: &AttrParams) -> syn::Result<TokenStream>;
}

impl AttrToTokens for Option<ZValueAttr> {
    fn attr_tokens(&self, attr_params: &AttrParams) -> syn::Result<TokenStream> {
        let &AttrParams {
            input,
            zenoh_pico,
            zvalue_ty,
            ..
        } = attr_params;
        let zvalue_trait: Path = parse_quote!(#zenoh_pico::zvalue::ZValue);
        let zvalue_impl =
            derive::impl_signature(&input, Some(&quote! { #zvalue_trait<#zvalue_ty> }));
        let from_impl =
            derive::impl_signature(&input, Some(&quote! { core::convert::From<#zvalue_ty> }));

        Ok(quote! {
            #zvalue_impl { }

            #from_impl {
                fn from(value: #zvalue_ty) -> Self {
                    Self(value)
                }
            }
        })
    }
}

impl AttrToTokens for Option<ZMoveAttr> {
    fn attr_tokens(&self, attr_params: &AttrParams) -> syn::Result<TokenStream> {
        let &AttrParams {
            base,
            input,
            zenoh_pico,
            zenoh_pico_sys,
            zvalue_ty,
            ..
        } = attr_params;

        let [zmove_ty_cfg, zmove_fn_cfg, zdrop_fn_cfg] = self
            .as_ref()
            .map(|z| [&z.ty, &z.zfn, &z.drop_zfn].map(Option::as_ref))
            .unwrap_or([None, None, None]);
        let zmove_ty = path_or_sys_default(
            zmove_ty_cfg,
            &format_ident!("z_moved_{base}_t"),
            &zenoh_pico_sys,
        )?;
        let zmove_fn = path_or_sys_default(
            zmove_fn_cfg,
            &format_ident!("z_{base}_move"),
            &zenoh_pico_sys,
        )?;
        let zdrop_fn = path_or_sys_default(
            zdrop_fn_cfg,
            &format_ident!("z_{base}_drop"),
            &zenoh_pico_sys,
        )?;

        let zown_trait: Path = parse_quote!(#zenoh_pico::zvalue::ZOwn);
        let zown_impl =
            derive::impl_signature(&input, Some(&quote! { #zown_trait<#zvalue_ty, #zmove_ty> }));
        let drop_impl = derive::impl_signature(&input, Some(&quote! { core::ops::Drop }));

        Ok(quote! {
            #zown_impl {
                fn zmove(mut self) -> *mut #zmove_ty {
                    unsafe { #zmove_fn(&mut self.0) }
                }
            }

            #drop_impl {
                fn drop(&mut self) {
                    unsafe { #zdrop_fn(#zmove_fn(&mut self.0)); }
                }
            }
        })
    }
}

impl AttrToTokens for Option<ZDefaultAttr> {
    fn attr_tokens(&self, attr_params: &AttrParams) -> syn::Result<TokenStream> {
        let &AttrParams { input, .. } = attr_params;

        let zdefault_call = self.as_ref().and_then(|z| z.zfn.as_ref()).map(|zfn| {
            quote! {
                unsafe {
                    #zfn(&mut zvalue);
                }
            }
        });
        let default_impl = derive::impl_signature(&input, Some(&quote! { core::default::Default }));

        Ok(quote! {
            #default_impl {
                fn default() -> Self {
                    let mut zvalue = Default::default();
                    #zdefault_call
                    Self(zvalue)
                }
            }
        })
    }
}

impl AttrToTokens for Option<ZLoanAttr> {
    fn attr_tokens(&self, attr_params: &AttrParams) -> syn::Result<TokenStream> {
        let &AttrParams {
            base,
            input,
            zenoh_pico,
            zenoh_pico_sys,
            zvalue_ty,
            ..
        } = attr_params;

        let Some(zloan) = self else {
            return Ok(Default::default());
        };

        let zloan_ty = path_or_sys_default(
            zloan.ty.as_ref(),
            &format_ident!("z_loaned_{base}_t"),
            &zenoh_pico_sys,
        )?;
        let zloan_fn = path_or_sys_default(
            zloan.zfn.as_ref(),
            &format_ident!("z_{base}_loan"),
            &zenoh_pico_sys,
        )?;

        let zloan_trait: Path = parse_quote!(#zenoh_pico::zvalue::ZLoan);
        let zloan_impl = derive::impl_signature(
            &input,
            Some(&quote! { #zloan_trait<#zvalue_ty, #zloan_ty> }),
        );

        let mut tokens = quote! {
            #zloan_impl {
                fn zloan(&self) -> *const #zloan_ty {
                    unsafe { #zloan_fn(&self.0) }
                }
            }
        };

        let Some(zloan_mut) = zloan.mutable.as_ref() else {
            return Ok(tokens);
        };

        let zloan_fn_mut = path_or_sys_default(
            zloan_mut.zfn.as_ref(),
            &format_ident!("z_{base}_loan_mut"),
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
        Ok(tokens)
    }
}

pub fn impl_zown(mut input: ZValueInput, config: ZOwnConfig) -> syn::Result<TokenStream> {
    let mut tokens = TokenStream::new();
    let base = config.base;
    let zenoh_pico = zenoh_pico_path()?;
    let zenoh_pico_sys = zenoh_pico_sys_path()?;

    let zvalue_ty = zvalue_type_path(config.zvalue.as_ref(), &base, &zenoh_pico_sys)?;
    tokens.extend(input.transform_struct(&zvalue_ty)?);

    let attr_params = AttrParams {
        base: &base,
        input: &input,
        zenoh_pico: &zenoh_pico,
        zenoh_pico_sys: &zenoh_pico_sys,
        zvalue_ty: &zvalue_ty,
    };

    tokens.extend(config.zvalue.attr_tokens(&attr_params)?);
    tokens.extend(config.zmove.attr_tokens(&attr_params)?);
    tokens.extend(config.zdefault.attr_tokens(&attr_params)?);
    tokens.extend(config.zloan.attr_tokens(&attr_params)?);

    Ok(tokens)
}
