use std::{fmt::Display, ops::Deref};

use darling::FromMeta;
use macro_utils::derive::DeriveInputExtensions;
use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, format_ident, quote};
use syn::{Data, DataStruct, DeriveInput, Error, Fields, Ident, Path, parse::Parse, parse_quote};

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
    family: InternalTypeFamily,
}

impl Display for TypeBase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.name.fmt(f)
    }
}

impl TypeBase {
    fn ident(&self) -> Ident {
        let name = &self.name;
        match self.family {
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
    value_ty: Option<Path>,
    #[darling(default = impl_trait_attr_default)]
    impl_from: bool,
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
    owned_ty: Option<Path>,
    owned_attr: Option<Ident>,
    moved_ty: Option<Path>,
    move_zfn: Option<Path>,
    drop_zfn: Option<Path>,
    #[darling(default = impl_trait_attr_default)]
    impl_drop: bool,
    #[darling(default = impl_trait_attr_default)]
    impl_from: bool,
}

#[derive(FromMeta, Default)]
#[darling(default, from_word = default_meta_from_word)]
struct ZTakeAttr {
    base: Option<TypeBase>,
    take_zfn: Option<Path>,
    #[darling(default = impl_trait_attr_default)]
    impl_try_from: bool,
}

#[derive(FromMeta, Default)]
#[darling(default, from_word = default_meta_from_word)]
pub struct ZClosureAttr {
    base: Option<TypeBase>,
    callback_ty: Option<Path>,
    init_zfn: Option<Path>,
}

#[derive(FromMeta, Default)]
#[darling(default, derive_syn_parse)]
pub struct ZWrapConfig {
    base: Option<TypeBase>,
    zvalue: Option<ZValueAttr>,
    zown: Option<ZOwnAttr>,
    // zloan: Option<ZLoanAttr>,
    // zloan_mut: Option<ZLoanMutAttr>,
    ztake: Option<ZTakeAttr>,
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

struct ZWrapParams {
    base: Option<TypeBase>,
    zenoh_pico: Path,
    zenoh_pico_sys: Path,
}

impl ZWrapParams {
    pub fn merged_base<'a>(&'a self, base: Option<&'a TypeBase>) -> Option<&'a TypeBase> {
        base.or(self.base.as_ref())
    }

    pub fn path_or_sys_default<IdentCreator>(
        &self,
        path: Option<&Path>,
        sys_ident_creator: IdentCreator,
        base: Option<&TypeBase>,
    ) -> syn::Result<Path>
    where
        IdentCreator: FnOnce(&TypeBase) -> Ident,
    {
        path.map(ToTokens::into_token_stream)
            .or_else(|| {
                self.merged_base(base).map(sys_ident_creator).map(|i| {
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
        if self.impl_from {
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

        let family = self
            .base
            .as_ref()
            .map(|b| b.family)
            .unwrap_or(InternalTypeFamily::default());
        match family {
            InternalTypeFamily::Primitive => {
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
                .merged_base(self.base.as_ref())
                .map(|b| b.family)
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

                fn inspect_zowned<F, T>(&self, f: F) -> T
                where
                    F: FnOnce(&Self::OwnedValue) -> T,
                {
                    let mut zowned = Self::OwnedValue::default();
                    zowned.#owned_attr = self.0;
                    f(&mut zowned)
                }

                fn inspect_zowned_mut<F, T>(&mut self, f: F) -> T
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

                fn zmove(mut self) -> *mut Self::MovedValue {
                    let moved = self.inspect_zowned_mut(|z| unsafe { #move_zfn(z) });
                    ::std::mem::forget(self);
                    moved
                }

                fn zdrop(&mut self) {
                    self.inspect_zowned_mut(|z| unsafe { #drop_zfn(#move_zfn(z)) });
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

impl ZWrapAttrTokens for ZTakeAttr {
    fn to_tokens(&self, input: &ZWrapInput, params: &ZWrapParams) -> syn::Result<TokenStream> {
        let &ZWrapParams { zenoh_pico, .. } = &params;

        let take_zfn = params.path_or_sys_default(
            self.take_zfn.as_ref(),
            |b| format_ident!("z_{b}_take_from_loaned"),
            self.base.as_ref(),
        )?;

        let ztake_trait: Path = parse_quote!(#zenoh_pico::zvalue::ZTake);
        let ztake_impl = &input.impl_signature(Some(&ztake_trait.to_token_stream()));
        let zvalue_trait: Path = parse_quote!(#zenoh_pico::zvalue::ZValue);
        let zown_trait: Path = parse_quote!(#zenoh_pico::zvalue::ZOwn);
        let zenoh_error_ty: Path = parse_quote!(#zenoh_pico::result::ZenohError);
        let result_ty: Path = parse_quote!(::core::result::Result);
        let zresult_trait: Path = parse_quote!(#zenoh_pico::result::IntoZenohResult);

        let mut tokens = quote! {
            #ztake_impl {
                type Error = #zenoh_error_ty;

                fn ztake(loan_mut: *mut <Self as #zvalue_trait>::Value) -> #result_ty<Self, Self::Error> {
                    use #zresult_trait as _;

                    let mut value = <Self as #zvalue_trait>::uninitialized();
                    <Self as #zown_trait>
                        ::inspect_zowned_mut(
                            &mut value,
                            |z| unsafe { #take_zfn(z, loan_mut).into_zresult() },
                        )
                        .map(|_| value)
                }
            }
        };
        if self.impl_try_from {
            let try_from_impl = &input.impl_signature(Some(
                &quote! { ::core::convert::TryFrom<*mut <Self as #zvalue_trait>::Value> },
            ));

            tokens.extend(quote! {
                #try_from_impl {
                    type Error = <Self as #ztake_trait>::Error;

                    fn try_from(value: *mut <Self as #zvalue_trait>::Value) -> #result_ty<Self, Self::Error> {
                        <Self as #ztake_trait>::ztake(value)
                    }
                }
            });
        }

        Ok(tokens)
    }
}

impl ZWrapAttrTokens for ZClosureAttr {
    fn to_tokens(&self, input: &ZWrapInput, params: &ZWrapParams) -> syn::Result<TokenStream> {
        let &ZWrapParams { zenoh_pico, .. } = &params;

        let callback_ty = params.path_or_sys_default(
            self.callback_ty.as_ref(),
            |b| {
                let name = b.name.strip_prefix("closure_").unwrap_or(&b.name);
                let base = TypeBase {
                    name: name.to_owned(),
                    ..b.clone()
                };
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

        Ok(quote! {
            #zclosure_impl {
                type CallbackValue = #callback_ty;

                fn from_callback<T>(
                    callback: unsafe extern "C" fn(*mut Self::CallbackValue, *mut #cvoid_ty),
                    context: ::core::option::Option<#arc_ty<T>>,
                ) -> #zenoh_result_ty<Self> {
                    use #zresult_trait as _;

                    // Rc reference for the closure to ensure it lives the whole time.
                    // Atomic because zenoh could use multiple threads.
                    let context_ptr = context
                            .map(|arc| #arc_ty::into_raw(arc) as *mut #cvoid_ty)
                            .unwrap_or(std::ptr::null_mut());

                    unsafe extern "C" fn drop_context<T>(ptr: *mut #cvoid_ty) {
                        if !ptr.is_null() {
                            drop(#arc_ty::<T>::from_raw(ptr as *const T));
                        }
                    }

                    let mut value = <Self as #zvalue_trait>::uninitialized();
                    <Self as #zown_trait>
                        ::inspect_zowned_mut(
                            &mut value,
                            |z| unsafe {
                                #init_zfn(
                                    z,
                                    Some(callback),
                                    Some(drop_context::<T>),
                                    context_ptr,
                                ).into_zresult()
                            },
                        )
                        .map(|_| value)
                }
            }
        })
    }
}

pub fn zwrap(input: ZWrapInput, config: ZWrapConfig) -> syn::Result<TokenStream> {
    let mut tokens = TokenStream::new();
    let base = config.base;
    let zenoh_pico = zenoh_pico_path()?;
    let zenoh_pico_sys = zenoh_pico_sys_path()?;

    let params = ZWrapParams {
        base,
        zenoh_pico,
        zenoh_pico_sys,
    };

    tokens.extend(input.to_tokens(&params)?);

    let attributes: [Option<&dyn ZWrapAttrTokens>; _] = [
        config.zvalue.as_ref().map(|a| a as _),
        config.zown.as_ref().map(|a| a as _),
        // config.zloan.as_ref().map(|a| a as _),
        // config.zloan_mut.as_ref().map(|a| a as _),
        config.ztake.as_ref().map(|a| a as _),
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
