use std::ops::Deref;

use darling::FromMeta;
use macro_utils::derive::DeriveInputExtensions;
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

#[derive(FromMeta, Default, Clone)]
#[darling(default)]
struct ZCallbackAttr {
    ty: Option<Path>,
    zfn: Option<Path>,
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

#[derive(FromMeta)]
#[darling(derive_syn_parse)]
pub struct ZClosureConfig {
    base: String,
    #[darling(default)]
    zvalue: Option<ZValueAttr>,
    #[darling(default)]
    zmove: Option<ZMoveAttr>,
    #[darling(default)]
    zloan: Option<ZLoanAttr>,
    #[darling(default)]
    zcallback: Option<ZCallbackAttr>,
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
    default_sys_ident: &Ident,
    zenoh_pico_sys: &Path,
) -> syn::Result<Path> {
    path_or_sys_default(
        zvalue.and_then(|z| z.ty.as_ref()),
        default_sys_ident,
        zenoh_pico_sys,
    )
}

fn fn_base(base: &str, fn_prefix: Option<&str>) -> String {
    [
        fn_prefix.map(|p| format!("{p}_")).unwrap_or_default(),
        base.to_string(),
    ]
    .concat()
}

fn ztype_ident(name: &str, base: &str) -> Ident {
    format_ident!("z_{name}_{base}_t")
}

fn zfn_ident(name: &str, base: &str, fn_prefix: Option<&str>) -> Ident {
    let fn_base = self::fn_base(base, fn_prefix);
    format_ident!("z_{fn_base}_{name}")
}

struct AttrParams<'a> {
    base: &'a str,
    input: &'a ZValueInput,
    zenoh_pico: &'a Path,
    zenoh_pico_sys: &'a Path,
    zvalue_ty: &'a Path,
    fn_prefix: Option<&'a str>,
}

impl AttrParams<'_> {
    pub fn ztype_ident(&self, name: &str) -> Ident {
        self::ztype_ident(name, self.base)
    }

    pub fn zfn_ident(&self, name: &str) -> Ident {
        self::zfn_ident(name, self.base, self.fn_prefix)
    }
}

trait AttrPaths {
    fn attr_paths(&self, attr_params: &AttrParams) -> syn::Result<TokenStream>;
}

impl AttrPaths for Option<ZValueAttr> {
    fn attr_paths(&self, attr_params: &AttrParams) -> syn::Result<TokenStream> {
        let &AttrParams {
            input,
            zenoh_pico,
            zvalue_ty,
            ..
        } = attr_params;
        let zvalue_trait: Path = parse_quote!(#zenoh_pico::zvalue::ZValue);
        let zvalue_impl = &input.impl_signature(Some(&zvalue_trait.to_token_stream()));
        let from_impl = &input.impl_signature(Some(&quote! { core::convert::From<#zvalue_ty> }));

        Ok(quote! {
            #zvalue_impl {
                type Value = #zvalue_ty;
            }

            #from_impl {
                fn from(value: #zvalue_ty) -> Self {
                    Self(value)
                }
            }
        })
    }
}

impl AttrPaths for Option<ZMoveAttr> {
    fn attr_paths(&self, attr_params: &AttrParams) -> syn::Result<TokenStream> {
        let &AttrParams {
            input,
            zenoh_pico,
            zenoh_pico_sys,
            ..
        } = attr_params;

        let zmove_ty = path_or_sys_default(
            self.as_ref().and_then(|z| z.ty.as_ref()),
            &attr_params.ztype_ident("moved"),
            &zenoh_pico_sys,
        )?;
        let zmove_fn = path_or_sys_default(
            self.as_ref().and_then(|z| z.zfn.as_ref()),
            &attr_params.zfn_ident("move"),
            &zenoh_pico_sys,
        )?;
        let zdrop_fn = path_or_sys_default(
            self.as_ref().and_then(|z| z.drop_zfn.as_ref()),
            &attr_params.zfn_ident("drop"),
            &zenoh_pico_sys,
        )?;

        let zown_trait: Path = parse_quote!(#zenoh_pico::zvalue::ZOwn);
        let zown_impl = &input.impl_signature(Some(&zown_trait.to_token_stream()));
        let drop_impl = &input.impl_signature(Some(&quote! { core::ops::Drop }));

        Ok(quote! {
            #zown_impl {
                type MovedValue = #zmove_ty;

                fn zmove(mut self) -> *mut Self::MovedValue {
                    let zmoved = unsafe { #zmove_fn(&mut self.0) };
                    ::std::mem::forget(self); // prevent Drop call, ownership is transferred
                    zmoved
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

impl AttrPaths for Option<ZDefaultAttr> {
    fn attr_paths(&self, attr_params: &AttrParams) -> syn::Result<TokenStream> {
        let &AttrParams {
            input, zvalue_ty, ..
        } = attr_params;

        let zdefault_call = self.as_ref().and_then(|z| z.zfn.as_ref()).map(|zfn| {
            quote! {
                unsafe {
                    #zfn(&mut zvalue);
                }
            }
        });
        let default_impl = &input.impl_signature(Some(&quote! { core::default::Default }));

        Ok(quote! {
            #default_impl {
                fn default() -> Self {
                    let mut zvalue = #zvalue_ty::default();
                    #zdefault_call
                    Self(zvalue)
                }
            }
        })
    }
}

impl AttrPaths for Option<ZLoanAttr> {
    fn attr_paths(&self, attr_params: &AttrParams) -> syn::Result<TokenStream> {
        let &AttrParams {
            input,
            zenoh_pico,
            zenoh_pico_sys,
            ..
        } = attr_params;

        let Some(zloan) = self else {
            return Ok(Default::default());
        };

        let zloan_ty = path_or_sys_default(
            zloan.ty.as_ref(),
            &attr_params.ztype_ident("loaned"),
            &zenoh_pico_sys,
        )?;
        let zloan_fn = path_or_sys_default(
            zloan.zfn.as_ref(),
            &attr_params.zfn_ident("loan"),
            &zenoh_pico_sys,
        )?;

        let zloan_trait: Path = parse_quote!(#zenoh_pico::zvalue::ZLoan);
        let zloan_impl = &input.impl_signature(Some(&zloan_trait.to_token_stream()));

        let mut tokens = quote! {
            #zloan_impl {
                type LoanedValue = #zloan_ty;

                fn zloan(&self) -> *const Self::LoanedValue {
                    unsafe { #zloan_fn(&self.0) }
                }
            }
        };

        let Some(zloan_mut) = zloan.mutable.as_ref() else {
            return Ok(tokens);
        };

        let zloan_fn_mut = path_or_sys_default(
            zloan_mut.zfn.as_ref(),
            &attr_params.zfn_ident("loan_mut"),
            &zenoh_pico_sys,
        )?;

        let zloan_mut_trait: Path = parse_quote!(#zenoh_pico::zvalue::ZLoanMut);
        let zloan_mut_impl = &input.impl_signature(Some(&zloan_mut_trait.to_token_stream()));

        tokens.extend(quote! {
            #zloan_mut_impl {
                fn zloan_mut(&mut self) -> *mut <Self as #zloan_trait>::LoanedValue {
                    unsafe { #zloan_fn_mut(&mut self.0) }
                }
            }
        });
        Ok(tokens)
    }
}

impl AttrPaths for Option<ZCallbackAttr> {
    fn attr_paths(&self, attr_params: &AttrParams) -> syn::Result<TokenStream> {
        let &AttrParams {
            base,
            input,
            zenoh_pico,
            zenoh_pico_sys,
            zvalue_ty,
            ..
        } = attr_params;

        let zcallback_ty = path_or_sys_default(
            self.as_ref().and_then(|z| z.ty.as_ref()),
            &format_ident!("z_loaned_{}_t", base.strip_prefix("closure_").unwrap_or(base)),
            &zenoh_pico_sys,
        )?;
        let zcallback_fn = path_or_sys_default(
            self.as_ref().and_then(|z| z.zfn.as_ref()),
            &format_ident!("z_{base}"),
            &zenoh_pico_sys,
        )?;

        let zclosure_trait: Path = parse_quote!(#zenoh_pico::zvalue::ZClosure);
        let zclosure_impl = &input.impl_signature(Some(&zclosure_trait.to_token_stream()));
        let zenoh_result_ty: Path = parse_quote!(#zenoh_pico::result::ZenohResult);
        let zresult_trait: Path = parse_quote!(#zenoh_pico::result::ZResult);
        let zenoh_drop_ty: Path = parse_quote!(#zenoh_pico_sys::z_closure_drop_callback_t);
        let cvoid_ty: Path = parse_quote!(::core::ffi::c_void);

        Ok(quote! {
            #zclosure_impl {
                type CallbackValue = #zcallback_ty;

                fn from_callback<T>(
                    callback: unsafe extern "C" fn(*mut Self::CallbackValue, *mut #cvoid_ty),
                    drop: #zenoh_drop_ty,
                    context: ::core::option::Option<&mut T>,
                ) -> #zenoh_result_ty<Self> {
                    use #zresult_trait as _;

                    let mut closure = #zvalue_ty::default();
                    let context_ptr = context
                        .map(|i| i as *mut _ as *mut #cvoid_ty)
                        .unwrap_or(::core::ptr::null_mut());
                    unsafe {
                        #zcallback_fn(
                            &mut closure,
                            Some(callback),
                            drop,
                            context_ptr,
                        ).zresult(())?;
                    }
                    #zenoh_result_ty::Ok(Self(closure))
                }
            }
        })
    }
}

pub fn zown(mut input: ZValueInput, config: ZOwnConfig) -> syn::Result<TokenStream> {
    let mut tokens = TokenStream::new();
    let base = config.base;
    let zenoh_pico = zenoh_pico_path()?;
    let zenoh_pico_sys = zenoh_pico_sys_path()?;

    let zvalue_ty = zvalue_type_path(
        config.zvalue.as_ref(),
        &ztype_ident("owned", &base),
        &zenoh_pico_sys,
    )?;
    tokens.extend(input.transform_struct(&zvalue_ty)?);

    let attr_params = AttrParams {
        base: &base,
        input: &input,
        zenoh_pico: &zenoh_pico,
        zenoh_pico_sys: &zenoh_pico_sys,
        zvalue_ty: &zvalue_ty,
        fn_prefix: None,
    };

    tokens.extend(config.zvalue.attr_paths(&attr_params)?);
    tokens.extend(config.zmove.attr_paths(&attr_params)?);
    tokens.extend(config.zdefault.attr_paths(&attr_params)?);
    tokens.extend(config.zloan.attr_paths(&attr_params)?);

    Ok(tokens)
}

pub fn zview(mut input: ZValueInput, config: ZViewConfig) -> syn::Result<TokenStream> {
    let mut tokens = TokenStream::new();
    let base = config.base;
    let zenoh_pico = zenoh_pico_path()?;
    let zenoh_pico_sys = zenoh_pico_sys_path()?;

    let zvalue_ty = zvalue_type_path(
        config.zvalue.as_ref(),
        &ztype_ident("view", &base),
        &zenoh_pico_sys,
    )?;
    tokens.extend(input.transform_struct(&zvalue_ty)?);

    let attr_params = AttrParams {
        base: &base,
        input: &input,
        zenoh_pico: &zenoh_pico,
        zenoh_pico_sys: &zenoh_pico_sys,
        zvalue_ty: &zvalue_ty,
        fn_prefix: Some("view"),
    };

    tokens.extend(config.zvalue.attr_paths(&attr_params)?);
    tokens.extend(config.zdefault.attr_paths(&attr_params)?);
    tokens.extend(config.zloan.attr_paths(&attr_params)?);

    Ok(tokens)
}

pub fn zclosure(mut input: ZValueInput, config: ZClosureConfig) -> syn::Result<TokenStream> {
    let mut tokens = TokenStream::new();
    let base = format!("closure_{}", config.base);
    let zenoh_pico = zenoh_pico_path()?;
    let zenoh_pico_sys = zenoh_pico_sys_path()?;

    let zvalue_ty = zvalue_type_path(
        config.zvalue.as_ref(),
        &ztype_ident("owned", &base),
        &zenoh_pico_sys,
    )?;
    tokens.extend(input.transform_struct(&zvalue_ty)?);

    let attr_params = AttrParams {
        base: &base,
        input: &input,
        zenoh_pico: &zenoh_pico,
        zenoh_pico_sys: &zenoh_pico_sys,
        zvalue_ty: &zvalue_ty,
        fn_prefix: None,
    };

    tokens.extend(config.zvalue.attr_paths(&attr_params)?);
    tokens.extend(config.zmove.attr_paths(&attr_params)?);
    tokens.extend(config.zloan.attr_paths(&attr_params)?);
    tokens.extend(config.zcallback.attr_paths(&attr_params)?);

    Ok(tokens)
}
