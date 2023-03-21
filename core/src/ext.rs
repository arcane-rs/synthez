//! Extensions for [`syn`] types.

use proc_macro2::Span;
use sealed::sealed;
use syn::{punctuated::Punctuated, token};

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

    /// Parses [`syn::Fields::Unnamed`] from this consumed [`syn::Data::Struct`]
    /// and returns owning iterator over them.
    ///
    /// # Errors
    ///
    /// - If this [`syn::Data`] is not a [`syn::Data::Struct`].
    /// - If this [`syn::Data::Struct`] doesn't consist of
    ///   [`syn::Fields::Unnamed`].
    fn unnamed_fields(
        self,
    ) -> syn::Result<Punctuated<syn::Field, token::Comma>>;

    /// Parses [`syn::Fields::Unnamed`] from this borrowed [`syn::Data::Struct`]
    /// and returns referencing iterator over them.
    ///
    /// # Errors
    ///
    /// - If this [`syn::Data`] is not a [`syn::Data::Struct`].
    /// - If this [`syn::Data::Struct`] doesn't consist of
    ///   [`syn::Fields::Unnamed`].
    fn unnamed_fields_ref(
        &self,
    ) -> syn::Result<&Punctuated<syn::Field, token::Comma>>;
}

#[sealed]
impl Data for syn::Data {
    fn named_fields(self) -> syn::Result<Punctuated<syn::Field, token::Comma>> {
        match self {
            Self::Struct(data) => match data.fields {
                syn::Fields::Named(f) => Ok(f.named),
                syn::Fields::Unit | syn::Fields::Unnamed(_) => {
                    Err(syn::Error::new_spanned(
                        &data.fields,
                        "expected named struct fields only",
                    ))
                }
            },
            Self::Enum(data) => Err(syn::Error::new_spanned(
                data.enum_token,
                "expected struct only",
            )),
            Self::Union(data) => Err(syn::Error::new_spanned(
                data.union_token,
                "expected struct only",
            )),
        }
    }

    fn named_fields_ref(
        &self,
    ) -> syn::Result<&Punctuated<syn::Field, token::Comma>> {
        match self {
            Self::Struct(data) => match &data.fields {
                syn::Fields::Named(f) => Ok(&f.named),
                syn::Fields::Unit | syn::Fields::Unnamed(_) => {
                    Err(syn::Error::new_spanned(
                        &data.fields,
                        "expected named struct fields only",
                    ))
                }
            },
            Self::Enum(data) => Err(syn::Error::new_spanned(
                data.enum_token,
                "expected struct only",
            )),
            Self::Union(data) => Err(syn::Error::new_spanned(
                data.union_token,
                "expected struct only",
            )),
        }
    }

    fn unnamed_fields(
        self,
    ) -> syn::Result<Punctuated<syn::Field, token::Comma>> {
        match self {
            Self::Struct(data) => match data.fields {
                syn::Fields::Unnamed(f) => Ok(f.unnamed),
                syn::Fields::Unit | syn::Fields::Named(_) => {
                    Err(syn::Error::new_spanned(
                        &data.fields,
                        "expected unnamed struct fields only",
                    ))
                }
            },
            Self::Enum(data) => Err(syn::Error::new_spanned(
                data.enum_token,
                "expected struct only",
            )),
            Self::Union(data) => Err(syn::Error::new_spanned(
                data.union_token,
                "expected struct only",
            )),
        }
    }

    fn unnamed_fields_ref(
        &self,
    ) -> syn::Result<&Punctuated<syn::Field, token::Comma>> {
        match self {
            Self::Struct(data) => match &data.fields {
                syn::Fields::Unnamed(f) => Ok(&f.unnamed),
                syn::Fields::Unit | syn::Fields::Named(_) => {
                    Err(syn::Error::new_spanned(
                        &data.fields,
                        "expected unnamed struct fields only",
                    ))
                }
            },
            Self::Enum(data) => Err(syn::Error::new_spanned(
                data.enum_token,
                "expected struct only",
            )),
            Self::Union(data) => Err(syn::Error::new_spanned(
                data.union_token,
                "expected struct only",
            )),
        }
    }
}

/// Extension of a [`syn::Ident`](struct@syn::Ident).
#[sealed]
pub trait Ident {
    /// Creates a new [`syn::Ident`] out of the given string value with a
    /// [`Span::call_site`].
    ///
    /// [`syn::Ident`]: struct@syn::Ident
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
