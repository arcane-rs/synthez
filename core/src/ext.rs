use proc_macro2::Span;
use sealed::sealed;
use syn::{punctuated::Punctuated, spanned::Spanned as _, token};

/// Extension of an [`Option`] providing common function widely used by this
/// crate.
#[sealed]
pub trait OptionExt {
    /// Type of the value wrapped into this [`Option`].
    type Inner;

    /// Transforms the `Option<T>` into a `Result<(), E>`, mapping `None` to
    /// `Ok(())` and `Some(v)` to `Err(err(v))`.
    ///
    /// # Errors
    ///
    /// If `self` is [`None`].
    fn none_or_else<E>(
        self,
        err: impl FnOnce(Self::Inner) -> E,
    ) -> Result<(), E>;
}

#[sealed]
impl<T> OptionExt for Option<T> {
    type Inner = T;

    #[inline]
    fn none_or_else<E>(
        self,
        err: impl FnOnce(Self::Inner) -> E,
    ) -> Result<(), E> {
        match self {
            Some(v) => Err(err(v)),
            None => Ok(()),
        }
    }
}

#[sealed]
pub trait DataExt {
    fn named_fields(self) -> syn::Result<Punctuated<syn::Field, token::Comma>>;

    fn named_fields_ref(
        &self,
    ) -> syn::Result<&Punctuated<syn::Field, token::Comma>>;
}

#[sealed]
impl DataExt for syn::Data {
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

#[sealed]
pub trait IdentExt {
    #[must_use]
    fn new_on_call_site(ident: &str) -> syn::Ident;
}

#[sealed]
impl IdentExt for syn::Ident {
    #[inline]
    fn new_on_call_site(string: &str) -> Self {
        Self::new(string, Span::call_site())
    }
}
