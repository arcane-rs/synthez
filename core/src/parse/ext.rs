//! Extensions for [`syn::parse`] types.
//!
//! [`syn::parse`]: mod@syn::parse

use std::{any::TypeId, iter};

use proc_macro2::Span;
use sealed::sealed;
use syn::{
    parse::Parse,
    punctuated::Punctuated,
    token::{self, Token},
};

/// Extension of a [`syn::parse::ParseBuffer`] providing common function widely
/// used by this crate for parsing.
#[sealed]
pub trait ParseBuffer {
    /// Tries to parse `T` as the next [`Token`].
    ///
    /// Doesn't move [`ParseBuffer`]'s cursor if there is no `T`.
    ///
    /// # Errors
    ///
    /// If `T` fails to be parsed.
    fn try_parse<T: Default + Parse + Token>(&self) -> syn::Result<Option<T>>;

    /// Checks whether the next [`Token`] is `T`.
    ///
    /// Doesn't move [`ParseBuffer`]'s cursor.
    #[must_use]
    fn is_next<T: Default + Token>(&self) -> bool;

    /// Parses the next [`Token`] as [`syn::Ident`] _allowing_ Rust keywords,
    /// while default [`Parse`] implementation for [`syn::Ident`] disallows
    /// them.
    ///
    /// Always moves [`ParseBuffer`]'s cursor.
    ///
    /// # Errors
    ///
    /// If [`syn::Ident`] fails to be parsed.
    ///
    /// [`syn::Ident`]: struct@syn::Ident
    fn parse_any_ident(&self) -> syn::Result<syn::Ident>;

    /// Parses the next [`Token`] as [`syn::Ident`] _allowing_ Rust keywords,
    /// while default [`Parse`] implementation for [`syn::Ident`] disallows
    /// them. Drops the parsed [`Token`] in-place.
    ///
    /// Always moves [`ParseBuffer`]'s cursor.
    ///
    /// # Errors
    ///
    /// If [`syn::Ident`] fails to be parsed.
    ///
    /// [`syn::Ident`]: struct@syn::Ident
    fn skip_any_ident(&self) -> syn::Result<()> {
        self.parse_any_ident().map(drop)
    }

    /// Parses the wrapped (in a wrapper `W`) [`Token`]s as `T` [`Punctuated`]
    /// with a `P` separator.
    ///
    /// Always moves [`ParseBuffer`]'s cursor.
    ///
    /// # Errors
    ///
    /// If parsing [`Punctuated`] `T` wrapped into `W` fails.
    fn parse_wrapped_and_punctuated<T, W, P>(
        &self,
    ) -> syn::Result<Punctuated<T, P>>
    where
        T: Parse,
        W: Default + Token + AcceptedWrapper + 'static,
        P: Default + Parse + Token;

    /// Checks whether the next [`Token`] is a wrapper `W` and if yes, then
    /// parses the wrapped [`Token`]s as `T` [`Punctuated`] with a `P`
    /// separator. Otherwise, parses just `T`.
    ///
    /// Always moves [`ParseBuffer`]'s cursor.
    ///
    /// # Errors
    ///
    /// If either parsing [`Punctuated`] `T` wrapped into `W`, or parsing just
    /// `T`, fails.
    fn parse_maybe_wrapped_and_punctuated<T, W, P>(
        &self,
    ) -> syn::Result<Punctuated<T, P>>
    where
        T: Parse,
        W: Default + Token + AcceptedWrapper + 'static,
        P: Default + Parse + Token;

    /// Checks whether the next [`Token`] is a wrapper `W` and if yes, then
    /// parses the wrapped [`Token`]s as `T` [`Punctuated`] with a `P`
    /// separator. Otherwise, parses just `T` following the [`token::Eq`].
    ///
    /// Always moves [`ParseBuffer`]'s cursor.
    ///
    /// # Errors
    ///
    /// If either parsing [`Punctuated`] `T` wrapped into `W`, or parsing just
    /// `T` following the [`token::Eq`], fails.
    ///
    /// [`token::Eq`]: struct@token::Eq
    fn parse_eq_or_wrapped_and_punctuated<T, W, P>(
        &self,
    ) -> syn::Result<Punctuated<T, P>>
    where
        T: Parse,
        W: Default + Token + AcceptedWrapper + 'static,
        P: Default + Parse + Token;
}

#[sealed]
impl<'buf> ParseBuffer for syn::parse::ParseBuffer<'buf> {
    fn try_parse<T: Default + Parse + Token>(&self) -> syn::Result<Option<T>> {
        self.is_next::<T>().then(|| self.parse()).transpose()
    }

    fn is_next<T: Default + Token>(&self) -> bool {
        self.lookahead1().peek(|_| T::default())
    }

    fn parse_any_ident(&self) -> syn::Result<syn::Ident> {
        <syn::Ident as syn::ext::IdentExt>::parse_any(self)
    }

    fn parse_wrapped_and_punctuated<T, W, P>(
        &self,
    ) -> syn::Result<Punctuated<T, P>>
    where
        T: Parse,
        W: Default + Token + AcceptedWrapper + 'static,
        P: Default + Parse + Token,
    {
        let inner;
        if TypeId::of::<W>() == TypeId::of::<token::Bracket>() {
            _ = syn::bracketed!(inner in self);
        } else if TypeId::of::<W>() == TypeId::of::<token::Brace>() {
            _ = syn::braced!(inner in self);
        } else if TypeId::of::<W>() == TypeId::of::<token::Paren>() {
            _ = syn::parenthesized!(inner in self);
        } else {
            return Err(syn::Error::new(
                Span::call_site(),
                "`ParseBufferExt::parse_wrapped_and_punctuated` supports only \
                 brackets, braces and parentheses as wrappers.",
            ));
        }
        Punctuated::parse_terminated(&inner)
    }

    fn parse_maybe_wrapped_and_punctuated<T, W, P>(
        &self,
    ) -> syn::Result<Punctuated<T, P>>
    where
        T: Parse,
        W: Default + Token + AcceptedWrapper + 'static,
        P: Default + Parse + Token,
    {
        Ok(if self.is_next::<W>() {
            self.parse_wrapped_and_punctuated::<T, W, P>()?
        } else {
            iter::once(self.parse::<T>()?).collect()
        })
    }

    fn parse_eq_or_wrapped_and_punctuated<T, W, P>(
        &self,
    ) -> syn::Result<Punctuated<T, P>>
    where
        T: Parse,
        W: Default + Token + AcceptedWrapper + 'static,
        P: Default + Parse + Token,
    {
        Ok(if self.is_next::<W>() {
            self.parse_wrapped_and_punctuated::<T, W, P>()?
        } else {
            _ = self.parse::<token::Eq>()?;
            iter::once(self.parse::<T>()?).collect()
        })
    }
}

/// Trait marking [`Token`] types accepted by
/// [`parse_wrapped_and_punctuated()`][0] method (and similar) as a wrapper.
///
/// [0]: ParseBuffer::parse_wrapped_and_punctuated
#[sealed]
pub trait AcceptedWrapper {}

#[sealed]
impl AcceptedWrapper for token::Bracket {}

#[sealed]
impl AcceptedWrapper for token::Brace {}

#[sealed]
impl AcceptedWrapper for token::Paren {}
