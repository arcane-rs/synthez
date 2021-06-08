use std::{any::TypeId, iter};

use sealed::sealed;
use syn::{
    parse::{Parse, ParseBuffer},
    punctuated::Punctuated,
    token::{self, Token},
};

/// Extension of a [`syn::ParseBuffer`] providing common function widely used by
/// this crate for parsing.
#[sealed]
pub trait ParseBufferExt {
    /// Tries to parse `T` as the next [`Token`].
    ///
    /// Doesn't move [`ParseStream`]'s cursor if there is no `T`.
    fn try_parse<T: Default + Parse + Token>(&self) -> syn::Result<Option<T>>;

    /// Checks whether the next [`Token`] is `T`.
    ///
    /// Doesn't move [`ParseStream`]'s cursor.
    #[must_use]
    fn is_next<T: Default + Token>(&self) -> bool;

    /// Parses the next [`Token`] as [`syn::Ident`] _allowing_ Rust keywords,
    /// while default [`Parse`] implementation for [`syn::Ident`] disallows
    /// keywords.
    ///
    /// Always moves [`ParseStream`]'s cursor.
    fn parse_any_ident(&self) -> syn::Result<syn::Ident>;

    /// Parses the next [`Token`] as [`syn::Ident`] _allowing_ Rust keywords,
    /// while default [`Parse`] implementation for [`syn::Ident`] disallows
    /// keywords, and drops it in-place.
    ///
    /// Always moves [`ParseStream`]'s cursor.
    #[inline]
    fn skip_any_ident(&self) -> syn::Result<()> {
        self.parse_any_ident().map(drop)
    }

    /// Parses the wrapped (in a wrapper `W`) [`Token`]s as `T` [`Punctuated`]
    /// with `P`.
    ///
    /// Always moves [`ParseStream`]'s cursor.
    fn parse_wrapped_and_punctuated<T, W, P>(
        &self,
    ) -> syn::Result<Punctuated<T, P>>
    where
        T: Parse,
        W: Default + Token + 'static,
        P: Default + Parse + Token;

    /// Checks whether the next [`Token`] is a wrapper `W` and if yes, then
    /// parses the wrapped [`Token`]s as `T` [`Punctuated`] with `P`. Otherwise,
    /// parses just `T`.
    ///
    /// Always moves [`ParseStream`]'s cursor.
    fn parse_maybe_wrapped_and_punctuated<T, W, P>(
        &self,
    ) -> syn::Result<Punctuated<T, P>>
    where
        T: Parse,
        W: Default + Token + 'static,
        P: Default + Parse + Token;

    /// Checks whether the next [`Token`] is a wrapper `W` and if yes, then
    /// parses the wrapped [`Token`]s as `T` [`Punctuated`] with `P`. Otherwise,
    /// parses just `T` following the [`token::Eq`].
    ///
    /// Always moves [`ParseStream`]'s cursor.
    fn parse_eq_or_wrapped_and_punctuated<T, W, P>(
        &self,
    ) -> syn::Result<Punctuated<T, P>>
    where
        T: Parse,
        W: Default + Token + 'static,
        P: Default + Parse + Token;
}

#[sealed]
impl<'buf> ParseBufferExt for ParseBuffer<'buf> {
    #[inline]
    fn try_parse<T: Default + Parse + Token>(&self) -> syn::Result<Option<T>> {
        self.is_next::<T>().then(|| self.parse()).transpose()
    }

    #[inline]
    fn is_next<T: Default + Token>(&self) -> bool {
        self.lookahead1().peek(|_| T::default())
    }

    #[inline]
    fn parse_any_ident(&self) -> syn::Result<syn::Ident> {
        <syn::Ident as syn::ext::IdentExt>::parse_any(self)
    }

    fn parse_wrapped_and_punctuated<T, W, P>(
        &self,
    ) -> syn::Result<Punctuated<T, P>>
    where
        T: Parse,
        W: Default + Token + 'static,
        P: Default + Parse + Token,
    {
        let inner;
        if TypeId::of::<W>() == TypeId::of::<token::Bracket>() {
            let _ = syn::bracketed!(inner in self);
        } else if TypeId::of::<W>() == TypeId::of::<token::Brace>() {
            let _ = syn::braced!(inner in self);
        } else if TypeId::of::<W>() == TypeId::of::<token::Paren>() {
            let _ = syn::parenthesized!(inner in self);
        } else {
            unimplemented!(
                "ParseBufferExt::parse_wrapped_and_punctuated supports only \
                 brackets, braces and parentheses as wrappers.",
            );
        }
        Punctuated::parse_terminated(&inner)
    }

    fn parse_maybe_wrapped_and_punctuated<T, W, P>(
        &self,
    ) -> syn::Result<Punctuated<T, P>>
    where
        T: Parse,
        W: Default + Token + 'static,
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
        W: Default + Token + 'static,
        P: Default + Parse + Token,
    {
        Ok(if self.is_next::<W>() {
            self.parse_wrapped_and_punctuated::<T, W, P>()?
        } else {
            self.parse::<token::Eq>()?;
            iter::once(self.parse::<T>()?).collect()
        })
    }
}
