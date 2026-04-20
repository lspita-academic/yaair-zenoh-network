use darling::{ast, Error, FromDeriveInput, FromField, FromMeta, Result};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_quote, Path, Type};

use crate::zenoh_pico_path;

#[derive(FromMeta)]
struct ZDropArgs {
    zfn: Path,
}

#[derive(FromMeta)]
struct ZMoveArgs {
    ty: Path,
    zfn: Path,
}

#[derive(FromMeta)]
struct ZDefaultArgs {
    zfn: Path,
}

#[derive(FromMeta)]
struct ZLoanArgs {
    ty: Path,
    zfn: Path,
    #[darling(default)]
    zfn_mut: Option<Path>,
}

#[derive(FromField)]
struct ZValueField {
    ty: Type,
}

#[derive(FromDeriveInput)]
#[darling(attributes(zvalue), supports(struct_tuple))]
pub struct ZValueReceiver {
    ident: syn::Ident,
    generics: syn::Generics,
    data: ast::Data<(), ZValueField>,
    zdrop: ZDropArgs,
    zmove: ZMoveArgs,
    #[darling(default)]
    zdefault: Option<ZDefaultArgs>,
    #[darling(default)]
    zloan: Option<ZLoanArgs>,
}

impl ZValueReceiver {
    fn zvalue_type(&self) -> Result<&Type> {
        let fields = self
            .data
            .as_ref()
            .take_struct()
            .expect("supports(struct_tuple) guarantees a struct variant");

        match fields.fields.as_slice() {
            [field] => Ok(&field.ty),
            _ => Err(Error::custom(
                "Zenoh type wrapper must have exactly one unnamed field",
            )),
        }
    }

    fn impl_for(&self, trait_tokens: Option<TokenStream>) -> TokenStream {
        let ident = &self.ident;
        let (impl_generics, type_generics, where_clause) = self.generics.split_for_impl();
        let trait_part = trait_tokens.map(|t| quote! { #t for }).unwrap_or_default();
        quote! { impl #impl_generics #trait_part #ident #type_generics #where_clause }
    }
}

impl ToTokens for ZValueReceiver {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let zenoh_pico_path = match zenoh_pico_path() {
            Ok(p) => p,
            Err(e) => {
                tokens.extend(e.to_compile_error());
                return;
            }
        };

        let zvalue_type = match self.zvalue_type() {
            Ok(t) => t,
            Err(e) => {
                tokens.extend(e.write_errors());
                return;
            }
        };

        let zdrop_fn = &self.zdrop.zfn;
        let zmove_ty = &self.zmove.ty;
        let zmove_fn = &self.zmove.zfn;

        let zvalue_trait: Path = parse_quote!(#zenoh_pico_path::zvalue::ZValue);
        let zvalue_impl = self.impl_for(Some(quote! { #zvalue_trait<#zvalue_type, #zmove_ty> }));
        let from_impl = self.impl_for(Some(quote! { core::convert::From<#zvalue_type> }));
        let drop_impl = self.impl_for(Some(quote! { core::ops::Drop }));

        tokens.extend(quote! {
            #zvalue_impl {
                fn zmove(mut self) -> *mut #zmove_ty {
                    unsafe { #zmove_fn(&mut self.0) }
                }
            }

            #from_impl {
                fn from(value: #zvalue_type) -> Self {
                    Self(value)
                }
            }

            #drop_impl {
                fn drop(&mut self) {
                    unsafe { #zdrop_fn(#zmove_fn(&mut self.0)) };
                }
            }
        });

        let zdefault_init = match &self.zdefault {
            Some(zd) => {
                let zd_fn = &zd.zfn;
                quote! { unsafe { #zd_fn(&mut zvalue); } }
            }
            None => TokenStream::new(),
        };

        let default_impl = self.impl_for(Some(quote! { core::default::Default }));
        tokens.extend(quote! {
            #default_impl {
                fn default() -> Self {
                    let mut zvalue = Default::default();
                    #zdefault_init
                    Self(zvalue)
                }
            }
        });

        if let Some(zloan) = &self.zloan {
            let zloan_trait: Path = parse_quote!(#zenoh_pico_path::zvalue::ZLoan);
            let zloan_ty = &zloan.ty;
            let zloan_fn = &zloan.zfn;

            let zloan_impl = self.impl_for(Some(
                quote! { #zloan_trait<#zvalue_type, #zmove_ty, #zloan_ty> },
            ));

            tokens.extend(quote! {
                #zloan_impl {
                    fn zloan(&self) -> *const #zloan_ty {
                        unsafe { #zloan_fn(&self.0) }
                    }
                }
            });

            if let Some(zloan_fn_mut) = &zloan.zfn_mut {
                let zloan_mut_trait: Path = parse_quote!(#zenoh_pico_path::zvalue::ZLoanMut);
                let zloan_mut_impl = self.impl_for(Some(
                    quote! { #zloan_mut_trait<#zvalue_type, #zmove_ty, #zloan_ty> },
                ));

                tokens.extend(quote! {
                    #zloan_mut_impl {
                        fn zloan_mut(&mut self) -> *mut #zloan_ty {
                            unsafe { #zloan_fn_mut(&mut self.0) }
                        }
                    }
                });
            }
        }
    }
}
