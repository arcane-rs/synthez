//! Extensions for [`syn`] types.

use proc_macro2::Span;
use sealed::sealed;
use syn::{punctuated::Punctuated, spanned::Spanned as _, token};

/// Extension of a [`syn::Data`].
#[sealed]
pub trait Data {
    /// Parses [`syn::Fields::Named`] from this consumed [`syn::Data::Struct`]
    /// and returns owning iterator over them.
    ///
    /// # Errors
    ///
    /// - If this [`syn::Data`] is not a [`syn::Data::Struct`].
    /// - If this [`syn::Data::Struct`] doesn't consist of
    ///   [`syn::Fields::Named`].
    fn named_fields(self) -> syn::Result<Punctuated<syn::Field, token::Comma>>;

    /// Parses [`syn::Fields::Named`] from this borrowed [`syn::Data::Struct`]
    /// and returns referencing iterator over them.
    ///
    /// # Errors
    ///
    /// - If this [`syn::Data`] is not a [`syn::Data::Struct`].
    /// - If this [`syn::Data::Struct`] doesn't consist of
    ///   [`syn::Fields::Named`].
    fn named_fields_ref(
        &self,
    ) -> syn::Result<&Punctuated<syn::Field, token::Comma>>;
}

#[sealed]
impl Data for syn::Data {
    fn named_fields(self) -> syn::Result<Punctuated<syn::Field, token::Comma>> {
        match self {
            syn::Data::Struct(data) => match data.fields {
                syn::Fields::Named(f) => Ok(f.named),
                fields => Err(syn::Error::new(
                    fields.span(),
                    "expected named struct fields only",
                )),
            },
            syn::Data::Enum(data) => Err(syn::Error::new(
                data.enum_token.span(),
                "expected struct only",
            )),
            syn::Data::Union(data) => Err(syn::Error::new(
                data.union_token.span(),
                "expected struct only",
            )),
        }
    }

    fn named_fields_ref(
        &self,
    ) -> syn::Result<&Punctuated<syn::Field, token::Comma>> {
        match self {
            syn::Data::Struct(data) => match &data.fields {
                syn::Fields::Named(f) => Ok(&f.named),
                fields => Err(syn::Error::new(
                    fields.span(),
                    "expected named struct fields only",
                )),
            },
            syn::Data::Enum(data) => Err(syn::Error::new(
                data.enum_token.span(),
                "expected struct only",
            )),
            syn::Data::Union(data) => Err(syn::Error::new(
                data.union_token.span(),
                "expected struct only",
            )),
        }
    }
}

/// Extension of a [`syn::Ident`].
#[sealed]
pub trait Ident {
    /// Creates a new [`syn::Ident`] out of the given string value with a
    /// [`Span::call_site`].
    #[must_use]
    fn new_on_call_site(ident: &str) -> syn::Ident;
}

#[sealed]
impl Ident for syn::Ident {
    #[inline]
    fn new_on_call_site(string: &str) -> Self {
        Self::new(string, Span::call_site())
    }
}
