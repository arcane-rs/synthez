//! Batteries for [`syn::parse`](mod@syn::parse).

pub mod attr;
pub mod attrs;
pub mod err;
pub mod ext;

#[doc(inline)]
pub use self::{attrs::Attrs, ext::ParseBuffer as BufferExt};
