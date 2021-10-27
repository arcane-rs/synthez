//! Batteries for parsing a single attribute.

use crate::Spanning;

/// Lookups for the standard Rust `#[doc]` attributes in the given
/// [`syn::Attribute`]s and parses their text as [`String`] with a little
/// normalization.
///
/// # Errors
///
/// - If parsing text from `#[doc]` attribute fails.
/// - If `#[doc]` doesn't contain text.
pub fn doc_string(
    attrs: &[syn::Attribute],
) -> syn::Result<Option<Spanning<String>>> {
    let mut span = None;
    let mut out = String::new();

    for a in super::attrs::filter_by_name("doc", attrs) {
        if let syn::Meta::NameValue(item) = a.parse_meta()? {
            if let syn::Lit::Str(lit) = item.lit {
                if span.is_none() {
                    span = Some(lit.span());
                }

                let s = lit.value();
                let s = s.strip_prefix(' ').unwrap_or(&s).trim_end();
                if s.ends_with('\\') {
                    out.push_str(s.trim_end_matches('\\'));
                    out.push(' ');
                } else {
                    out.push_str(s);
                    out.push('\n');
                }
            } else {
                return Err(syn::Error::new_spanned(
                    item.lit,
                    "`#[doc]` attribute can contain string literals only",
                ));
            }
        }
    }
    if !out.is_empty() {
        out.truncate(out.trim_end().len());
    }

    Ok(span.map(|s| Spanning::new(out, s)))
}

/// Lookups for the standard Rust `#[doc]` attributes in the given
/// [`syn::Attribute`]s and parses their text as [`syn::LitStr`] with a little
/// normalization.
///
/// # Errors
///
/// - If parsing text from `#[doc]` attribute fails.
/// - If `#[doc]` doesn't contain text.
///
/// [`syn::LitStr`]: struct@syn::LitStr
pub fn doc(attrs: &[syn::Attribute]) -> syn::Result<Option<syn::LitStr>> {
    doc_string(attrs).map(|opt| opt.map(Into::into))
}
