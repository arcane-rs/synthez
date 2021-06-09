use proc_macro2::Span;
use sealed::sealed;
use syn::{punctuated::Punctuated, spanned::Spanned as _, token};

#[sealed]
pub trait Data {
    fn named_fields(self) -> syn::Result<Punctuated<syn::Field, token::Comma>>;

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

#[sealed]
pub trait Ident {
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
