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
pub trait AsSpan {
    /// Returns the coerced [`Span`].
    #[must_use]
    fn as_span(&self) -> Span;
}

#[sealed]
impl AsSpan for Span {
    #[inline]
    fn as_span(&self) -> Self {
        *self
    }
}

#[sealed]
impl<T: Spanned> AsSpan for &T {
    #[inline]
    fn as_span(&self) -> Span {
        self.span()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Spanning<T: ?Sized> {
    span: Span,
    item: T,
}

impl<T> Spanning<T> {
    #[inline]
    #[must_use]
    pub fn new<S: AsSpan>(item: T, span: S) -> Self {
        Self { span: span.as_span(), item }
    }

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
