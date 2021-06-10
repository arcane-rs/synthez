//! Internal implementations of [`synthez`] crate. Refer to its documentation
//! for details.

#![cfg_attr(docsrs, feature(doc_cfg))]
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

pub mod codegen;
pub mod ext;
pub mod field;
pub mod has;
pub mod parse;
pub mod spanned;

pub use proc_macro2;
pub use quote::{self, ToTokens};
pub use syn;

#[doc(inline)]
pub use self::{
    ext::{Data as DataExt, Ident as IdentExt},
    field::Required,
    parse::{Attrs as ParseAttrs, BufferExt as ParseBufferExt},
    spanned::Spanning,
};
