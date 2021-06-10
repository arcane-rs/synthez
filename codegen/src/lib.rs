//! Internal codegen shim of [`synthez`] crate. Refer to its documentation for
//! details.

#![deny(
    nonstandard_style,
    rust_2018_idioms,
    rustdoc::broken_intra_doc_links,
    rustdoc::private_intra_doc_links,
    trivial_casts,
    trivial_numeric_casts
)]
#![forbid(unsafe_code)]
#![warn(
    deprecated_in_future,
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    unreachable_pub,
    unused_import_braces,
    unused_labels,
    unused_lifetimes,
    unused_qualifications,
    unused_results
)]

use proc_macro::TokenStream;
use synthez_core::codegen;

/// Deriving of [`synthez::ParseAttrs`] along with [`syn::parse::Parse`]
/// implementations to parse [`syn::Attribute`]s into a custom defined struct.
///
/// TODO: examples
#[proc_macro_derive(ParseAttrs, attributes(parse))]
pub fn derive_parse_attrs(input: TokenStream) -> TokenStream {
    syn::parse(input)
        .and_then(codegen::parse_attrs::derive)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

/// Deriving of a [`quote::ToTokens`] implementation.
///
/// TODO: examples
///
/// [`quote::ToTokens`]: synthez::quote::ToTokens
#[proc_macro_derive(ToTokens, attributes(to_tokens))]
pub fn derive_to_tokens(input: TokenStream) -> TokenStream {
    syn::parse(input)
        .and_then(|input| codegen::to_tokens::derive(&input))
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}
