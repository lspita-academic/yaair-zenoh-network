use std::{fmt::Display, ops::Deref};

use darling::FromMeta;
use macro_utils::derive::DeriveInputExtensions;
use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, format_ident, quote};
use syn::{
    Data, DataStruct, DeriveInput, Error, ExprPath, Fields, Ident, Path, TypePath, parse::Parse,
    parse_quote,
};

use crate::{zenoh_pico_path, zenoh_pico_sys_path};

fn default_meta_from_word<T: FromMeta>() -> darling::Result<T> {
    FromMeta::from_list(&[])
}

fn impl_trait_attr_default() -> bool {
    true
}

#[derive(FromMeta, Default, Clone, Copy)]
#[darling(default)]
pub enum InternalTypeFamily {
    #[default]
    Normal,
    Rc,
    Primitive,
}

#[derive(FromMeta, Clone)]
pub struct TypeBase {
    name: String,
    #[darling(default)]
    family: Option<InternalTypeFamily>,
}

impl Display for TypeBase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.name.fmt(f)
    }
}

impl TypeBase {
    #[allow(dead_code)]
    fn name(&self) -> &str {
        &self.name
    }

    fn family(&self) -> InternalTypeFamily {
        self.family.unwrap_or_default()
    }

    fn merge(&self, other: Option<&Self>) -> Self {
        other
            .map(|b| TypeBase {
                name: b.name.clone(),
                family: b.family.or(self.family),
            })
            .unwrap_or(self.clone())
    }

    fn rename<F: FnOnce(&str) -> String>(&self, f: F) -> Self {
        Self {
            name: f(&self.name),
            ..self.clone()
        }
    }

    fn ident(&self) -> Ident {
        let name = &self.name;
        match self.family() {
            InternalTypeFamily::Normal => format_ident!("_z_{name}_t"),
            InternalTypeFamily::Rc => format_ident!("_z_{name}_rc_t"),
            InternalTypeFamily::Primitive => format_ident!("z_{name}_t"),
        }
    }
}

#[derive(FromMeta, Default)]
#[darling(default, from_word = default_meta_from_word)]
pub struct ZValueAttr {
    base: Option<TypeBase>,
    value_ty: Option<TypePath>,
    #[darling(default = impl_trait_attr_default)]
    impl_from_value: bool,
    #[darling(default = || false)]
    impl_default: bool,
    #[darling(default = impl_trait_attr_default)]
    impl_deref: bool,
    #[darling(default = impl_trait_attr_default)]
    impl_deref_mut: bool,
}

#[derive(FromMeta, Default)]
#[darling(default, from_word = default_meta_from_word)]
struct ZOwnAttr {
    base: Option<TypeBase>,
    owned_ty: Option<TypePath>,
    owned_attr: Option<Ident>,
    moved_ty: Option<TypePath>,
    move_zfn: Option<ExprPath>,
    drop_zfn: Option<ExprPath>,
    #[darling(default = impl_trait_attr_default)]
    impl_drop: bool,
    #[darling(default = impl_trait_attr_default)]
    impl_from: bool,
}

#[derive(FromMeta, Default)]
#[darling(default, from_word = default_meta_from_word)]
struct ZCloneAttr {
    base: Option<TypeBase>,
    clone_zfn: Option<ExprPath>,
    #[darling(default = impl_trait_attr_default)]
    impl_clone: bool,
}

#[derive(FromMeta, Default)]
#[darling(default, from_word = default_meta_from_word)]
struct ZViewAttr {
    base: Option<TypeBase>,
    view_ty: Option<TypePath>,
    loan_zfn: Option<ExprPath>,
    loan_mut_zfn: Option<ExprPath>,
}

#[derive(FromMeta, Default)]
#[darling(default, from_word = default_meta_from_word)]
pub struct ZClosureAttr {
    base: Option<TypeBase>,
    callback_ty: Option<TypePath>,
    init_zfn: Option<ExprPath>,
}

#[derive(FromMeta, Default)]
#[darling(default, derive_syn_parse)]
pub struct ZWrapConfig {
    base: Option<TypeBase>,
    zvalue: Option<ZValueAttr>,
    zown: Option<ZOwnAttr>,
    zclone: Option<ZCloneAttr>,
    zview: Option<ZViewAttr>,
    zclosure: Option<ZClosureAttr>,
}

pub struct ZWrapInput(DeriveInput);

impl Deref for ZWrapInput {
    type Target = DeriveInput;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Parse for ZWrapInput {
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

struct ZWrapParams<'a> {
    base: Option<&'a TypeBase>,
    config: &'a ZWrapConfig,
    zenoh_pico: &'a Path,
    zenoh_pico_sys: &'a Path,
}

impl ZWrapParams<'_> {
    pub fn path_or_sys_default<P, IdentCreator>(
        &self,
        path: Option<&P>,
        sys_ident_creator: IdentCreator,
        base: Option<&TypeBase>,
    ) -> syn::Result<P>
    where
        P: Parse + ToTokens,
        IdentCreator: FnOnce(&TypeBase) -> Ident,
    {
        path.map(ToTokens::into_token_stream)
            .or_else(|| {
                self.base
                    .map(|b| b.merge(base))
                    .map(|b| sys_ident_creator(&b))
                    .map(|i| {
                        let zenoh_pico_sys = &self.zenoh_pico_sys;
                        quote! {#zenoh_pico_sys::#i}
                    })
            })
            .map(syn::parse2)
            .unwrap_or(Err(Error::new(
                Span::call_site(),
                "Either the base or the specific path should be set",
            )))
    }
}

trait ZWrapInputTokens {
    fn to_tokens(&self, params: &ZWrapParams) -> syn::Result<TokenStream>;
}

trait ZWrapAttrTokens {
    fn to_tokens(&self, input: &ZWrapInput, params: &ZWrapParams) -> syn::Result<TokenStream>;
}

impl ZWrapInputTokens for ZWrapInput {
    fn to_tokens(&self, params: &ZWrapParams) -> syn::Result<TokenStream> {
        let &ZWrapParams { zenoh_pico, .. } = &params;

        let zvalue_trait: Path = parse_quote!(#zenoh_pico::zvalue::ZValue);
        let struct_data = match &self.data {
            Data::Struct(struct_data) => struct_data.clone(),
            _ => unreachable!("ZValueInput guarantees unit struct"),
        };
        let data = Data::Struct(DataStruct {
            fields: Fields::Unnamed(syn::parse2(quote! {(<Self as #zvalue_trait>::Value)})?),
            ..struct_data
        });
        let input = DeriveInput {
            data,
            ..self.0.clone()
        };

        Ok(quote! {
            #[repr(transparent)]
            #input
        })
    }
}

impl ZWrapAttrTokens for ZValueAttr {
    fn to_tokens(&self, input: &ZWrapInput, params: &ZWrapParams) -> syn::Result<TokenStream> {
        let &ZWrapParams { zenoh_pico, .. } = &params;

        let value_ty = params.path_or_sys_default(
            self.value_ty.as_ref(),
            TypeBase::ident,
            self.base.as_ref(),
        )?;

        let zvalue_trait: Path = parse_quote!(#zenoh_pico::zvalue::ZValue);
        let zvalue_impl = &input.impl_signature(Some(&zvalue_trait.to_token_stream()));

        let mut tokens = quote! {
            #zvalue_impl {
                type Value = #value_ty;

                fn uninitialized() -> Self {
                    Self::from_zvalue(Self::Value::default())
                }

                fn from_zvalue(value: Self::Value) -> Self {
                    Self(value)
                }

                fn from_ptr<'a>(ptr: *const Self::Value) -> &'a Self {
                    unsafe { &*(ptr as *const Self) }
                }

                fn from_ptr_mut<'a>(ptr: *mut Self::Value) -> &'a mut Self {
                    unsafe { &mut *(ptr as *mut Self) }
                }

                fn zloan(&self) -> *const Self::Value {
                    &self.0
                }

                fn zloan_mut(&mut self) -> *mut Self::Value {
                    &mut self.0
                }
            }
        };
        if self.impl_from_value {
            let from_impl = &input.impl_signature(Some(
                &quote! { ::core::convert::From<<Self as #zvalue_trait>::Value> },
            ));

            tokens.extend(quote! {
                #from_impl {
                    fn from(value: <Self as #zvalue_trait>::Value) -> Self {
                        <Self as #zvalue_trait>::from_zvalue(value)
                    }
                }
            });
        }
        if self.impl_default {
            let default_impl = &input.impl_signature(Some(&quote! { ::core::default::Default }));

            tokens.extend(quote! {
                #default_impl {
                    fn default() -> Self {
                        <Self as #zvalue_trait>::uninitialized()
                    }
                }
            });
        }
        if self.impl_deref {
            let deref_impl = &input.impl_signature(Some(&quote! { ::core::ops::Deref }));

            tokens.extend(quote! {
                #deref_impl {
                    type Target = <Self as #zvalue_trait>::Value;

                    fn deref(&self) -> &Self::Target {
                        unsafe { &*<Self as #zvalue_trait>::zloan(self) }
                    }
                }
            });
        }
        if self.impl_deref_mut {
            let deref_mut_impl = &input.impl_signature(Some(&quote! { ::core::ops::DerefMut }));

            tokens.extend(quote! {
                #deref_mut_impl {
                    fn deref_mut(&mut self) -> &mut Self::Target {
                        unsafe { &mut *<Self as #zvalue_trait>::zloan_mut(self) }
                    }
                }
            });
        }

        Ok(tokens)
    }
}

impl ZWrapAttrTokens for ZOwnAttr {
    fn to_tokens(&self, input: &ZWrapInput, params: &ZWrapParams) -> syn::Result<TokenStream> {
        let &ZWrapParams { zenoh_pico, .. } = &params;

        match self.base.as_ref().map(|b| b.family()) {
            Some(InternalTypeFamily::Primitive) => {
                return Err(Error::new(
                    Span::call_site(),
                    "Cannot implement ZOwn trait for primitive zenoh value",
                ));
            }
            _ => {}
        };

        let owned_ty = params.path_or_sys_default(
            self.owned_ty.as_ref(),
            |b| format_ident!("z_owned_{b}_t"),
            self.base.as_ref(),
        )?;
        let owned_attr = self.owned_attr.clone().unwrap_or(
            match params
                .base
                .map(|b| b.merge(self.base.as_ref()))
                .map(|b| b.family())
                .unwrap_or_default()
            {
                InternalTypeFamily::Normal => parse_quote!(_val),
                InternalTypeFamily::Rc => parse_quote!(_rc),
                InternalTypeFamily::Primitive => unreachable!("Excluded before"),
            },
        );
        let moved_ty = params.path_or_sys_default(
            self.moved_ty.as_ref(),
            |b| format_ident!("z_moved_{b}_t"),
            self.base.as_ref(),
        )?;
        let move_zfn = params.path_or_sys_default(
            self.move_zfn.as_ref(),
            |b| format_ident!("z_{b}_move"),
            self.base.as_ref(),
        )?;
        let drop_zfn = params.path_or_sys_default(
            self.drop_zfn.as_ref(),
            |b| format_ident!("z_{b}_drop"),
            self.base.as_ref(),
        )?;

        let zown_trait: Path = parse_quote!(#zenoh_pico::zvalue::ZOwn);
        let zvalue_trait: Path = parse_quote!(#zenoh_pico::zvalue::ZValue);
        let zown_impl = &input.impl_signature(Some(&zown_trait.to_token_stream()));

        let mut tokens = quote! {
            #zown_impl {
                type OwnedValue = #owned_ty;
                type MovedValue = #moved_ty;

                fn with_zowned<F, T>(&self, f: F) -> T
                where
                    F: FnOnce(&Self::OwnedValue) -> T,
                {
                    let mut zowned = Self::OwnedValue::default();
                    zowned.#owned_attr = self.0;
                    f(&mut zowned)
                }

                fn with_zowned_mut<F, T>(&mut self, f: F) -> T
                where
                    F: FnOnce(&mut Self::OwnedValue) -> T,
                {
                    let mut zowned = Self::OwnedValue::default();
                    zowned.#owned_attr = self.0;
                    let res = f(&mut zowned);
                    self.0 = zowned.#owned_attr;
                    res
                }

                fn from_zowned(value: Self::OwnedValue) -> Self {
                    <Self as #zvalue_trait>::from_zvalue(value.#owned_attr)
                }

                fn zmove(mut self) -> Self::MovedValue {
                    let moved = self.with_zowned_mut(|z| unsafe { *#move_zfn(z) });
                    ::std::mem::forget(self);
                    moved
                }

                fn zdrop(&mut self) {
                    self.with_zowned_mut(|z| unsafe { #drop_zfn(#move_zfn(z)) });
                }
            }
        };
        if self.impl_drop {
            let drop_impl = &input.impl_signature(Some(&quote! { ::core::ops::Drop }));

            tokens.extend(quote! {
                #drop_impl {
                    fn drop(&mut self) {
                        <Self as #zown_trait>::zdrop(self);
                    }
                }
            });
        };
        if self.impl_from {
            let from_impl = &input.impl_signature(Some(
                &quote! { ::core::convert::From<<Self as #zown_trait>::OwnedValue> },
            ));

            tokens.extend(quote! {
                #from_impl {
                    fn from(value: <Self as #zown_trait>::OwnedValue) -> Self {
                        <Self as #zown_trait>::from_zowned(value)
                    }
                }
            });
        }

        Ok(tokens)
    }
}

impl ZWrapAttrTokens for ZCloneAttr {
    fn to_tokens(&self, input: &ZWrapInput, params: &ZWrapParams) -> syn::Result<TokenStream> {
        let &ZWrapParams {
            config, zenoh_pico, ..
        } = &params;

        let zclone_trait: Path = parse_quote!(#zenoh_pico::zvalue::ZClone);
        let zclone_impl = &input.impl_signature(Some(&zclone_trait.to_token_stream()));
        let zvalue_trait: Path = parse_quote!(#zenoh_pico::zvalue::ZValue);

        let clone_impl = if config.zown.is_some() {
            let clone_zfn = params.path_or_sys_default(
                self.clone_zfn.as_ref(),
                |b| format_ident!("z_{b}_clone"),
                self.base.as_ref(),
            )?;
            let zown_trait: Path = parse_quote!(#zenoh_pico::zvalue::ZOwn);

            quote! {
                let mut value = <Self as #zvalue_trait>::uninitialized();
                <Self as #zown_trait>::with_zowned_mut(
                    &mut value,
                    |z| unsafe {
                        #clone_zfn(z, ptr);
                    },
                );
                value
            }
        } else {
            let clone_zfn = self
                .clone_zfn
                .clone()
                .unwrap_or_else(|| parse_quote!(::core::ptr::read_unaligned));
            quote! { <Self as #zvalue_trait>::from_zvalue(unsafe { #clone_zfn(ptr) }) }
        };

        let mut tokens = quote! {
            #zclone_impl {
                fn zclone(ptr: *const <Self as #zvalue_trait>::Value) -> Self {
                    #clone_impl
                }
            }
        };
        if self.impl_clone {
            let clone_impl = &input.impl_signature(Some(&quote! { ::core::clone::Clone }));

            tokens.extend(quote! {
                #clone_impl {
                    fn clone(&self) -> Self {
                        <Self as #zclone_trait>::zclone(<Self as #zvalue_trait>::zloan(self))
                    }
                }
            });
        }

        Ok(tokens)
    }
}

impl ZWrapAttrTokens for ZViewAttr {
    fn to_tokens(&self, input: &ZWrapInput, params: &ZWrapParams) -> syn::Result<TokenStream> {
        let &ZWrapParams { zenoh_pico, .. } = &params;

        let view_ty = params.path_or_sys_default(
            self.loan_zfn.as_ref(),
            |b| format_ident!("z_view_{b}_t"),
            self.base.as_ref(),
        )?;
        let loan_zfn = params.path_or_sys_default(
            self.loan_zfn.as_ref(),
            |b| format_ident!("z_view_{b}_loan"),
            self.base.as_ref(),
        )?;
        let loan_mut_zfn = params.path_or_sys_default(
            self.loan_zfn.as_ref(),
            |b| format_ident!("z_view_{b}_loan_mut"),
            self.base.as_ref(),
        )?;

        let zview_trait: Path = parse_quote!(#zenoh_pico::zvalue::ZView);
        let zview_impl = &input.impl_signature(Some(&zview_trait.to_token_stream()));
        let zvalue_trait: Path = parse_quote!(#zenoh_pico::zvalue::ZValue);

        let tokens = quote! {
            #zview_impl {
                type ViewValue = #view_ty;

                fn from_zview<'a>(value: Self::ViewValue) -> &'a Self {
                    <Self as #zvalue_trait>::from_ptr(unsafe { #loan_zfn(&value) })
                }
                fn from_zview_mut<'a>(mut value: Self::ViewValue) -> &'a mut Self {
                    <Self as #zvalue_trait>::from_ptr_mut(unsafe { #loan_mut_zfn(&mut value) })
                }
            }
        };

        Ok(tokens)
    }
}

impl ZWrapAttrTokens for ZClosureAttr {
    fn to_tokens(&self, input: &ZWrapInput, params: &ZWrapParams) -> syn::Result<TokenStream> {
        let &ZWrapParams { zenoh_pico, .. } = &params;

        let callback_ty = params.path_or_sys_default(
            self.callback_ty.as_ref(),
            |b| {
                let base = b.rename(|s| s.strip_prefix("closure_").unwrap_or(s).to_owned());
                base.ident()
            },
            self.base.as_ref(),
        )?;
        let init_zfn = params.path_or_sys_default(
            self.init_zfn.as_ref(),
            |b| format_ident!("z_{b}"),
            self.base.as_ref(),
        )?;

        let zclosure_trait: Path = parse_quote!(#zenoh_pico::zvalue::ZClosure);
        let zvalue_trait: Path = parse_quote!(#zenoh_pico::zvalue::ZValue);
        let zown_trait: Path = parse_quote!(#zenoh_pico::zvalue::ZOwn);
        let zclosure_impl = &input.impl_signature(Some(&zclosure_trait.to_token_stream()));
        let zenoh_result_ty: Path = parse_quote!(#zenoh_pico::result::ZenohResult);
        let zresult_trait: Path = parse_quote!(#zenoh_pico::result::IntoZenohResult);
        let arc_ty: Path = parse_quote!(::std::sync::Arc);
        let cvoid_ty: Path = parse_quote!(::core::ffi::c_void);
        let trasmute_fn: Path = parse_quote!(::core::mem::transmute);

        Ok(quote! {
            #zclosure_impl {
                type CallbackValue = #callback_ty;

                fn from_callback<T>(
                    callback: unsafe extern "C" fn(*const Self::CallbackValue, *const T),
                    context: ::core::option::Option<#arc_ty<T>>,
                ) -> #zenoh_result_ty<Self> {
                    use #zresult_trait as _;

                    // Rc reference for the closure to ensure it lives the whole time.
                    // Atomic because zenoh could use multiple threads.
                    // Caller must use mutexes if a mutable reference is needed.
                    let context_ptr = context
                        .map(|arc| #arc_ty::into_raw(arc))
                        .unwrap_or(std::ptr::null());

                    unsafe extern "C" fn drop_context<T>(ptr: *const T) {
                        if !ptr.is_null() {
                            drop(unsafe { #arc_ty::from_raw(ptr) });
                        }
                    }
                    // get sized pointer to be able to call trasmute
                    let drop_fn = drop_context::<T> as unsafe extern "C" fn(*const T);

                    let mut value = <Self as #zvalue_trait>::uninitialized();
                    <Self as #zown_trait>::with_zowned_mut(
                        &mut value,
                        |z| unsafe {
                            #init_zfn(
                                z,
                                #trasmute_fn(Some(callback)),
                                #trasmute_fn(Some(drop_fn)),
                                context_ptr as *mut #cvoid_ty,
                            ).into_zresult()
                        },
                    )?;
                    #zenoh_result_ty::<Self>::Ok(value)
                }
            }
        })
    }
}

pub fn zwrap(input: ZWrapInput, config: ZWrapConfig) -> syn::Result<TokenStream> {
    let mut tokens = TokenStream::new();
    let base = config.base.as_ref();
    let zenoh_pico = &zenoh_pico_path()?;
    let zenoh_pico_sys = &zenoh_pico_sys_path()?;

    let params = ZWrapParams {
        base,
        config: &config,
        zenoh_pico,
        zenoh_pico_sys,
    };

    tokens.extend(input.to_tokens(&params)?);

    let attributes: [Option<&dyn ZWrapAttrTokens>; _] = [
        config.zvalue.as_ref().map(|a| a as _),
        config.zown.as_ref().map(|a| a as _),
        config.zclone.as_ref().map(|a| a as _),
        config.zview.as_ref().map(|a| a as _),
        config.zclosure.as_ref().map(|a| a as _),
    ];
    for a in attributes {
        let attr_tokens = a
            .map(|a| a.to_tokens(&input, &params))
            .unwrap_or(Ok(TokenStream::new()))?;
        tokens.extend(attr_tokens);
    }

    Ok(tokens)
}
