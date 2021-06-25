//! Steroids for [`syn`], [`quote`] and [`proc_macro2`] crates.
//!
//! # Cargo features
//!
//! ### `full`
//!
//! Same as `full` feature of [`syn`] crate.
//!
//! Enables support of data structures for representing the syntax tree of all
//! valid Rust source code, including items and expressions.
//!
//! # Example of writing `proc_macro_derive`
//!
//! This is an example of how this library can be used to write a simplified
//! `proc_macro_derive` implemention for deriving [`From`] implementations.
//!
//! ```rust
//! # use std::{collections::HashMap, convert::TryFrom};
//! #
//! # use synthez::{proc_macro2::{Span, TokenStream}, quote::quote, syn};
//! use synthez::{DataExt as _, ParseAttrs, ToTokens};
//!
//! pub fn derive(input: syn::DeriveInput) -> syn::Result<TokenStream> {
//!     let attrs = Attrs::parse_attrs("from", &input)?;
//!     match (attrs.forward.is_some(), !attrs.custom.is_empty()) {
//!         (true, true) => Err(syn::Error::new_spanned(
//!             input,
//!             "`forward` and `on` arguments are mutually exclusive",
//!         )),
//!         (false, false) => Err(syn::Error::new_spanned(
//!             input,
//!             "either `forward` or `on` argument is expected",
//!         )),
//!
//!         // #[from(forward)]
//!         (true, _) => {
//!             if !matches!(&input.data, syn::Data::Struct(_)) {
//!                 return Err(syn::Error::new_spanned(
//!                     input,
//!                     "only tuple structs can forward-derive From",
//!                 ));
//!             }
//!             let fields = input.data.unnamed_fields()?;
//!             if fields.len() > 1 {
//!                 return Err(syn::Error::new_spanned(
//!                     fields,
//!                     "only single-field tuple structs can forward-derive \
//!                      From",
//!                 ));
//!             }
//!             let definition = ForwardDefinition {
//!                 ty: input.ident,
//!                 inner_ty: fields.into_iter().last().unwrap().ty,
//!             };
//!             Ok(quote! {
//!                 #definition
//!             })
//!         }
//!
//!         // #[from(on <type> = <func>)]
//!         (_, true) => {
//!             let definitions =
//!                 CustomDefinitions { ty: input.ident, funcs: attrs.custom };
//!             Ok(quote! {
//!                 #definitions
//!             })
//!         }
//!     }
//! }
//!
//! #[derive(Default, ParseAttrs)]
//! struct Attrs {
//!     #[parse(ident)]
//!     forward: Option<syn::Ident>,
//!     #[parse(map, arg = on)]
//!     custom: HashMap<syn::Type, syn::Expr>,
//! }
//!
//! #[derive(ToTokens)]
//! #[to_tokens(append(impl_from))]
//! struct ForwardDefinition {
//!     ty: syn::Ident,
//!     inner_ty: syn::Type,
//! }
//!
//! impl ForwardDefinition {
//!     fn impl_from(&self) -> TokenStream {
//!         let (ty, inner_ty) = (&self.ty, &self.inner_ty);
//!         quote! {
//!             impl<T> From<T> for #ty where #inner_ty: From<T> {
//!                 fn from(v: T) -> Self {
//!                     Self(v.into())
//!                 }
//!             }
//!         }
//!     }
//! }
//!
//! #[derive(ToTokens)]
//! #[to_tokens(append(impl_froms))]
//! struct CustomDefinitions {
//!     ty: syn::Ident,
//!     funcs: HashMap<syn::Type, syn::Expr>,
//! }
//!
//! impl CustomDefinitions {
//!     fn impl_froms(&self) -> TokenStream {
//!         let ty = &self.ty;
//!         // We sort here for tests below not failing due to undetermined
//!         // order only. Real implementation may omit this.
//!         let mut sorted = self.funcs.iter().collect::<Vec<_>>();
//!         sorted.sort_unstable_by(|(ty1, _), (ty2, _)| {
//!             quote!(#ty1).to_string().cmp(&quote!(#ty2).to_string())
//!         });
//!         let impls = sorted.into_iter().map(move |(from_ty, func)| {
//!             quote! {
//!                 impl From<#from_ty> for #ty {
//!                     fn from(v: #from_ty) -> Self {
//!                         #func(v)
//!                     }
//!                 }
//!             }
//!         });
//!         quote! { #( #impls )* }
//!     }
//! }
//!
//! # fn main() {
//! let input = syn::parse_quote! {
//!     #[derive(From)]
//!     #[from(forward)]
//!     struct Id(u64);
//! };
//! let output = quote! {
//!     impl<T> From<T> for Id where u64: From<T> {
//!         fn from(v: T) -> Self {
//!             Self(v.into())
//!         }
//!     }
//! };
//! assert_eq!(derive(input).unwrap().to_string(), output.to_string());
//!
//! let input = syn::parse_quote! {
//!     #[derive(From)]
//!     #[from(on bool = Self::parse_bool)]
//!     #[from(on u8 = from_u8_to_maybe)]
//!     enum Maybe {
//!         Yes,
//!         No,
//!     }
//! };
//! let output = quote! {
//!     impl From<bool> for Maybe {
//!         fn from(v: bool) -> Self {
//!             Self::parse_bool(v)
//!         }
//!     }
//!     impl From<u8> for Maybe {
//!         fn from(v: u8) -> Self {
//!             from_u8_to_maybe(v)
//!         }
//!     }
//! };
//! assert_eq!(derive(input).unwrap().to_string(), output.to_string());
//! # }
//! ```

#![deny(
    nonstandard_style,
    rust_2018_idioms,
    rustdoc::broken_intra_doc_links,
    rustdoc::private_intra_doc_links,
    trivial_casts,
    trivial_numeric_casts
)]
#![forbid(non_ascii_idents, unsafe_code)]
#![warn(
    deprecated_in_future,
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    unreachable_pub,
    unused_import_braces,
    unused_labels,
    unused_lifetimes,
    unused_qualifications,
    unused_results
)]

#[doc(inline)]
pub use synthez_codegen::ToTokens;
#[doc(inline)]
pub use synthez_core::{ext, field, has, spanned};
pub use synthez_core::{
    proc_macro2,
    quote::{self, ToTokens},
    syn,
};

#[doc(inline)]
pub use self::{
    ext::{Data as DataExt, Ident as IdentExt},
    field::Required,
    parse::{Attrs as ParseAttrs, BufferExt as ParseBufferExt},
    spanned::Spanning,
};

pub mod parse {
    //! Batteries for [`syn::parse`](mod@crate::syn::parse).

    #[doc(inline)]
    pub use synthez_core::parse::{attr, err, ext};

    #[doc(inline)]
    pub use self::{attrs::Attrs, ext::ParseBuffer as BufferExt};

    pub mod attrs {
        //! Machinery for parsing [`syn::Attribute`]s into a custom defined
        //! struct.
        //!
        //! [`syn::Attribute`]: crate::syn::Attribute

        #[doc(inline)]
        pub use synthez_codegen::ParseAttrs as Attrs;
        #[doc(inline)]
        pub use synthez_core::parse::attrs::*;
    }
}
