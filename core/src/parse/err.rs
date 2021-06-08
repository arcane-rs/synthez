//! Common errors of parsing.

use crate::spanned::AsSpan;

/// Creates a "duplicated attribute's argument" [`syn::Error`] pointing to the
/// given `span`.
#[inline]
#[must_use]
pub fn dup_attr_arg<S: AsSpan>(span: S) -> syn::Error {
    syn::Error::new(span.as_span(), "duplicated attribute's argument found")
}

/// Creates an "unknown attribute's argument" [`syn::Error`] for the given
/// `name` pointing to the given `span`.
#[must_use]
pub fn unknown_attr_arg<S: AsSpan>(span: S, name: &str) -> syn::Error {
    syn::Error::new(
        span.as_span(),
        format!("unknown `{}` attribute argument", name),
    )
}

/// Creates a "required attribute's argument" [`syn::Error`] for the given
/// `name` in the given [`Span`].
#[must_use]
pub fn required_attr_arg<S: AsSpan>(span: S, name: &str) -> syn::Error {
    syn::Error::new(
        span.as_span(),
        format!("`{}` attribute argument is required, but is absent", name),
    )
}

/// Creates an "expected followed by comma" [`syn::Error`] in the given
/// [`Span`].
#[inline]
#[must_use]
pub fn expected_followed_by_comma<S: AsSpan>(span: S) -> syn::Error {
    syn::Error::new(span.as_span(), "expected followed by `,`")
}
