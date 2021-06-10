//! Batteries for [`Span`] and [`syn::spanned`].

use std::{
    cmp::{Eq, PartialEq},
    ops::{Deref, DerefMut},
};

use proc_macro2::Span;
use sealed::sealed;
use syn::spanned::Spanned;

/// Helper coercion for [`Span`] and [`Spanned`] types to use in function
/// arguments.
#[sealed]
pub trait IntoSpan {
    /// Returns the coerced [`Span`].
    #[must_use]
    fn into_span(self) -> Span;
}

#[sealed]
impl IntoSpan for Span {
    #[inline]
    fn into_span(self) -> Self {
        self
    }
}

#[sealed]
impl<T: Spanned> IntoSpan for &T {
    #[inline]
    fn into_span(self) -> Span {
        self.span()
    }
}

/// Wrapper for non-[`Spanned`] types to hold their [`Span`].
#[derive(Clone, Copy, Debug)]
pub struct Spanning<T: ?Sized> {
    /// [`Span`] of the `item`.
    span: Span,

    /// Item the [`Span`] is held for.
    item: T,
}

impl<T> Spanning<T> {
    /// Creates a new [`Spanning`] `item` out of the given value and its
    /// [`Span`].
    #[inline]
    #[must_use]
    pub fn new<S: IntoSpan>(item: T, span: S) -> Self {
        Self { span: span.into_span(), item }
    }

    /// Destructures this [`Spanning`] wrapper returning the underlying value.
    #[inline]
    #[must_use]
    pub fn into_inner(self) -> T {
        self.item
    }
}

impl<T: ?Sized> Deref for Spanning<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.item
    }
}

impl<T: ?Sized> DerefMut for Spanning<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.item
    }
}

impl<T, V> PartialEq<Spanning<V>> for Spanning<T>
where
    T: PartialEq<V> + ?Sized,
    V: ?Sized,
{
    #[inline]
    fn eq(&self, other: &Spanning<V>) -> bool {
        self.item.eq(&other.item)
    }
}

impl<T: PartialEq + ?Sized> Eq for Spanning<T> {}

impl<T: ?Sized> Spanned for Spanning<T> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl From<Spanning<&str>> for syn::LitStr {
    #[inline]
    fn from(s: Spanning<&str>) -> Self {
        Self::new(s.item, s.span)
    }
}

impl From<Spanning<String>> for syn::LitStr {
    #[inline]
    fn from(s: Spanning<String>) -> Self {
        Self::new(&s.item, s.span)
    }
}
