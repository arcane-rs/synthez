//! Internal codegen shim of [`synthez`] crate. Refer to its documentation for
//! details.
//!
//! [`synthez`]: https://docs.rs/synthez

#![deny(
    nonstandard_style,
    rust_2018_idioms,
    rustdoc::broken_intra_doc_links,
    rustdoc::private_intra_doc_links,
    trivial_casts,
    trivial_numeric_casts
)]
#![forbid(non_ascii_idents, unsafe_code)]
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

/// Deriving of [`synthez::ParseAttrs`] along with a [`syn::parse::Parse`]
/// implementation to parse [`syn::Attribute`]s into a custom defined struct.
///
/// # Field requirements
///
/// Each field should be wrapped into a [`field::Container`] implementor, which
/// describes and influes the parsing logic. Use [`Required`]
/// [`field::Container`] in case your parsing logic demands mandatory specifying
/// of a value.
///
/// Type of the parsed valued (the one contained in a [`field::Container`]) must
/// implement [`Parse`] and [`Spanned`] (vital for compile-time error
/// reporting). You may use the [`Spanning`] wrapper in case it doesn't
/// implement the latest.
///
/// # Arguments
///
/// ## `ident`, `value`, `map` or `nested` (mandatory)
///
/// Defines kind of parsing for a struct field.
///
/// ```rust
/// # use std::collections::{HashMap, HashSet};
/// #
/// # use syn::parse_quote;
/// # use synthez::{ParseAttrs, Spanning};
/// #
/// #[derive(Debug, Default, ParseAttrs, PartialEq)]
/// struct MyAttrs {
///     /// Will parse only `#[my_attr(ident)]`.
///     #[parse(ident)]
///     ident: Option<syn::Ident>,
///
///     /// Will parse `#[my_attr(value = <expr>)]`, `#[my_attr(value(<expr>))]`
///     /// and `#[my_attr(value(<expr1>, <expr2>))]`.
///     #[parse(value)]
///     value: Vec<syn::Expr>,
///
///     /// Will parse `#[my_attr(value <lit>)]`, `#[my_attr(value(<lit>))]`
///     /// and `#[my_attr(value(<lit1>, <lit2>))]`.
///     #[parse(value(spaced))]
///     value_spaced: HashSet<syn::Lit>,
///
///     /// Will parse `#[my_attr(map <ident> = <type>)]` only.
///     #[parse(map)]
///     map: HashMap<syn::Ident, syn::Type>,
///
///     /// Will parse `#[my_attr(nested(<arg1>, <arg2>))]` only.
///     ///
///     /// Note, we use [`Box`] here only because of recursive structure.
///     #[parse(nested)]
///     nested: Option<Spanning<Box<MyAttrs>>>,
/// }
///
/// # fn main() {
/// let input: syn::DeriveInput = parse_quote! {
///     #[my_attr(ident)]
///     #[my_attr(value = 2 * 2, value_spaced "some")]
///     #[my_attr(map A = Option<u8>)]
///     #[my_attr(map B = syn::Result<()>)]
///     #[my_attr(nested(ident, value = "another"))]
///     struct Dummy;
/// };
/// let my_attrs = MyAttrs::parse_attrs("my_attr", &input);
///
/// let expected_nested = MyAttrs {
///     ident: Some(parse_quote!(ident)),
///     value: vec![parse_quote!("another")],
///     ..MyAttrs::default()
/// };
///
/// assert!(my_attrs.is_ok());
/// # let my_attrs = my_attrs.unwrap();
/// assert_eq!(my_attrs.ident, Some(parse_quote!(ident)));
/// assert_eq!(my_attrs.value, vec![parse_quote!(2 * 2)]);
/// assert!(my_attrs.value_spaced.contains(&parse_quote!("some")));
/// assert_eq!(my_attrs.map.len(), 2);
/// assert_eq!(my_attrs.map[&parse_quote!(A)], parse_quote!(Option<u8>));
/// assert_eq!(my_attrs.map[&parse_quote!(B)], parse_quote!(syn::Result<()>));
/// assert_eq!(*my_attrs.nested.unwrap().into_inner(), expected_nested);
/// # }
/// ```
///
/// Only one such argument can be chosen for a single field.
///
/// ```rust,compile_fail
/// # use synthez::ParseAttrs;
/// #
/// #[derive(Default, ParseAttrs)]
/// struct Wrong {
///     /// We cannot use two kinds of parsing simultaneously.
///     #[parse(ident, value)]
///     field: Option<syn::Ident>,
/// }
/// ```
///
/// ## `alias = <name>`, `aliases(<name1>, <name2>)` (optional)
///
/// Adds aliases for an attribute's argument in addition to its field ident.
///
/// ```rust
/// # use syn::parse_quote;
/// # use synthez::ParseAttrs;
/// #
/// #[derive(Default, ParseAttrs)]
/// struct MyAttrs {
///     #[parse(value, alias = value)]
///     #[parse(aliases(vals, values))]
///     val: Vec<syn::Lit>,
/// }
///
/// # fn main() {
/// let input: syn::DeriveInput = parse_quote! {
///     #[my_attr(val = "foo")]
///     #[my_attr(value = "bar")]
///     #[my_attr(vals(1, 2), values(3, 4))]
///     struct Dummy;
/// };
/// let my_attrs = MyAttrs::parse_attrs("my_attr", &input);
///
/// # assert!(my_attrs.is_ok());
/// # let my_attrs = my_attrs.unwrap();
/// assert_eq!(my_attrs.val.len(), 6);
/// # }
/// ```
///
/// ## `arg = <name>`, `args(<name1>, <name2>)` (optional)
///
/// Similar to `alias` argument, but excludes the field ident from possible
/// names of a parsed attribute's argument. Can be used with `alias` argument
/// simultaneously.
///
/// ```rust
/// # use syn::parse_quote;
/// # use synthez::ParseAttrs;
/// #
/// #[derive(Default, ParseAttrs)]
/// struct MyAttrs {
///     #[parse(value, arg = value)]
///     #[parse(args(vals, values))]
///     #[parse(alias = v_a_l)]
///     val: Vec<syn::Lit>,
/// }
///
/// # fn main() {
/// let input: syn::DeriveInput = parse_quote! {
///     #[my_attr(value = "foo")]
///     #[my_attr(vals(1, 2), values(3, 4))]
///     #[my_attr(v_a_l = "bar")]
///     struct Dummy;
/// };
/// let my_attrs = MyAttrs::parse_attrs("my_attr", &input);
///
/// # assert!(my_attrs.is_ok());
/// # let my_attrs = my_attrs.unwrap();
/// assert_eq!(my_attrs.val.len(), 6);
///
/// let wrong: syn::DeriveInput = parse_quote! {
///     #[my_attr(val = "foo")]
///     struct Dummy;
/// };
/// let my_attrs = MyAttrs::parse_attrs("my_attr", &wrong);
///
/// assert!(my_attrs.is_err());
/// # }
/// ```
///
/// ## `dedup = <strategy>` (optional)
///
/// Defines deduplication strategy for the repeated same values during parsing.
/// Can be one of the following:
/// - `unique` (default): disallows duplicates;
/// - `first`: takes first value and ignores subsequent ones;
/// - `last`: takes last value and ignores previous ones.
///
/// ```rust
/// # use syn::parse_quote;
/// # use synthez::ParseAttrs;
/// #
/// #[derive(Default, ParseAttrs)]
/// struct MyAttrs {
///     /// Picks last appeared [`syn::Ident`] in attributes.
///     #[parse(ident, dedup = last, alias = named)]
///     name: Option<syn::Ident>,
///
///     /// Picks first value of `lit = <lit>` argument.
///     #[parse(value, dedup = first)]
///     lit: Option<syn::LitStr>,
///
///     /// Allows only one of `args`.
///     #[parse(ident, dedup = unique, args(foo, bar, baz))]
///     field: Option<syn::Ident>,
/// }
///
/// # fn main() {
/// let input: syn::DeriveInput = parse_quote! {
///     #[my_attr(name, lit = "foo")]
///     #[my_attr(named, lit = "bar")]
///     #[my_attr(baz)]
///     struct Dummy;
/// };
/// let my_attrs = MyAttrs::parse_attrs("my_attr", &input);
///
/// # assert!(my_attrs.is_ok());
/// # let my_attrs = my_attrs.unwrap();
/// assert_eq!(my_attrs.name, Some(parse_quote!(named)));
/// assert_eq!(my_attrs.lit, Some(parse_quote!("foo")));
/// assert_eq!(my_attrs.field, Some(parse_quote!(baz)));
///
/// let wrong: syn::DeriveInput = parse_quote! {
///     #[my_attr(foo, bar)]
///     #[my_attr(baz)]
///     struct Dummy;
/// };
/// let my_attrs = MyAttrs::parse_attrs("my_attr", &wrong);
///
/// assert!(my_attrs.is_err());
/// # }
/// ```
///
/// ## `validate = <func>` (optional)
///
/// Allows to specify a function for additional validation of the parsed field
/// value. The signature of the function should be the following:
/// ```rust,ignore
/// fn(&FieldType) -> syn::Result<()>
/// ```
///
/// ```rust
/// # use proc_macro2::Span;
/// # use syn::parse_quote;
/// # use synthez::ParseAttrs;
/// #
/// #[derive(Default, ParseAttrs)]
/// struct MyAttrs {
///     #[parse(value, validate = not_foo)]
///     val: Option<syn::LitStr>,
/// }
///
/// fn not_foo(lit: &Option<syn::LitStr>) -> syn::Result<()> {
///     if lit.as_ref().map(syn::LitStr::value).as_deref() == Some("foo") {
///         Err(syn::Error::new(Span::call_site(), "'foo' is not allowed"))
///     } else {
///         Ok(())
///     }
/// }
///
/// # fn main() {
/// let wrong: syn::DeriveInput = parse_quote! {
///     #[my_attr(val = "foo")]
///     struct Dummy;
/// };
/// let my_attrs = MyAttrs::parse_attrs("my_attr", &wrong);
///
/// assert!(my_attrs.is_err());
/// # }
/// ```
///
/// ## `fallback = <func>` (optional)
///
/// Allows to specify a function producing a fallback value for the prased field
/// value. The signature of the function should be the following:
/// ```rust,ignore
/// fn(&mut FieldType, ParsedInputType) -> syn::Result<()>
/// ```
///
/// This fallback function is invoked every time the field is parsed, despite
/// the kind of values it contains, so it's responsibility of the fallback
/// function to determine whether applying fallback value is actually required.
///
/// Note, that this argument accepts expressions, so you may use
/// [`field::if_empty()`] in a combination with a parse function to receive the
/// required signature. In such case the parse function has a way more obvious
/// signature:
/// ```rust,ignore
/// fn(ParsedInputType) -> syn::Result<ValueType>
/// ```
///
/// ```rust
/// # use syn::parse_quote;
/// use synthez::{field, parse, ParseAttrs};
///
/// #[derive(Default, ParseAttrs)]
/// struct MyAttrs {
///     /// `fallback` will use doc comment as a value, if no `desc` argument is
///     /// provided.
///     #[parse(value, fallback = field::if_empty(parse::attr::doc))]
///     desc: Option<syn::LitStr>,
/// }
///
/// # fn main() {
/// let from_attr: syn::DeriveInput = parse_quote! {
///     /// bar
///     #[my_attr(desc = "foo")]
///     struct Dummy;
/// };
/// let my_attrs = MyAttrs::parse_attrs("my_attr", &from_attr);
///
/// # assert!(my_attrs.is_ok());
/// # let my_attrs = my_attrs.unwrap();
/// assert_eq!(my_attrs.desc, Some(parse_quote!("foo")));
///
/// let from_doc: syn::DeriveInput = parse_quote! {
///     /// bar
///     struct Dummy;
/// };
/// let my_attrs = MyAttrs::parse_attrs("my_attr", &from_doc);
///
/// # assert!(my_attrs.is_ok());
/// # let my_attrs = my_attrs.unwrap();
/// assert_eq!(my_attrs.desc, Some(parse_quote!("bar")));
/// # }
/// ```
///
/// [`field::Container`]: synthez_core::field::Container
/// [`field::if_empty()`]: synthez_core::field::if_empty
/// [`Parse`]: syn::parse::Parse
/// [`Required`]: synthez_core::Required
/// [`Spanned`]: syn::spanned::Spanned
/// [`Spanning`]: synthez_core::Spanning
/// [`synthez::ParseAttrs`]: synthez_core::ParseAttrs
#[proc_macro_derive(ParseAttrs, attributes(parse))]
pub fn derive_parse_attrs(input: TokenStream) -> TokenStream {
    syn::parse(input)
        .and_then(codegen::parse_attrs::derive)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

/// Deriving of a [`quote::ToTokens`] implementation.
///
/// # Arguments
///
/// ## `append` (mandatory)
///
/// Specifies methods to form [`ToTokens`]' output with.
///
/// ```rust
/// # use synthez::{proc_macro2::TokenStream, quote::quote, ToTokens};
/// #
/// #[derive(ToTokens)]
/// #[to_tokens(append(foo_tokens, baz_tokens))]
/// struct Dummy;
///
/// impl Dummy {
///     fn foo_tokens(&self) -> TokenStream {
///         quote! {
///             impl Foo for String {}
///         }
///     }
///
///     fn baz_tokens(&self) -> TokenStream {
///         quote! {
///             impl Baz for String {}
///         }
///     }
/// }
///
/// # fn main() {
/// let dummy = Dummy;
///
/// assert_eq!(
///     quote! { #dummy }.to_string(),
///     quote! {
///         impl Foo for String {}
///         impl Baz for String {}
///     }
///     .to_string(),
/// );
/// # }
/// ```
///
/// [`quote::ToTokens`]: synthez_core::quote::ToTokens
/// [`ToTokens`]: synthez_core::quote::ToTokens
#[proc_macro_derive(ToTokens, attributes(to_tokens))]
pub fn derive_to_tokens(input: TokenStream) -> TokenStream {
    syn::parse(input)
        .and_then(|input| codegen::to_tokens::derive(&input))
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}
