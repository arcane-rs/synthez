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

#[proc_macro_derive(ParseAttrs, attributes(parse))]
pub fn derive_parse_attrs(input: TokenStream) -> TokenStream {
    syn::parse(input)
        .and_then(codegen::parse_attrs::derive)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

#[proc_macro_derive(ToTokens, attributes(to_tokens))]
pub fn derive_to_tokens(input: TokenStream) -> TokenStream {
    syn::parse(input)
        .and_then(codegen::to_tokens::derive)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}
