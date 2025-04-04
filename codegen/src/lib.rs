#![cfg_attr(any(doc, test), doc = include_str!("../README.md"))]
#![cfg_attr(not(any(doc, test)), doc = env!("CARGO_PKG_NAME"))]
#![deny(nonstandard_style, rustdoc::all, trivial_casts, trivial_numeric_casts)]
#![forbid(non_ascii_idents, unsafe_code)]
#![warn(
    clippy::absolute_paths,
    clippy::allow_attributes,
    clippy::allow_attributes_without_reason,
    clippy::as_conversions,
    clippy::as_pointer_underscore,
    clippy::as_ptr_cast_mut,
    clippy::assertions_on_result_states,
    clippy::branches_sharing_code,
    clippy::cfg_not_test,
    clippy::clear_with_drain,
    clippy::clone_on_ref_ptr,
    clippy::collection_is_never_read,
    clippy::create_dir,
    clippy::dbg_macro,
    clippy::debug_assert_with_mut_call,
    clippy::decimal_literal_representation,
    clippy::default_union_representation,
    clippy::derive_partial_eq_without_eq,
    clippy::doc_include_without_cfg,
    clippy::empty_drop,
    clippy::empty_structs_with_brackets,
    clippy::equatable_if_let,
    clippy::empty_enum_variants_with_brackets,
    clippy::exit,
    clippy::expect_used,
    clippy::fallible_impl_from,
    clippy::filetype_is_file,
    clippy::float_cmp_const,
    clippy::fn_to_numeric_cast_any,
    clippy::get_unwrap,
    clippy::if_then_some_else_none,
    clippy::imprecise_flops,
    clippy::infinite_loop,
    clippy::iter_on_empty_collections,
    clippy::iter_on_single_items,
    clippy::iter_over_hash_type,
    clippy::iter_with_drain,
    clippy::large_include_file,
    clippy::large_stack_frames,
    clippy::let_underscore_untyped,
    clippy::literal_string_with_formatting_args,
    clippy::lossy_float_literal,
    clippy::map_err_ignore,
    clippy::map_with_unused_argument_over_ranges,
    clippy::mem_forget,
    clippy::missing_assert_message,
    clippy::missing_asserts_for_indexing,
    clippy::missing_const_for_fn,
    clippy::missing_docs_in_private_items,
    clippy::module_name_repetitions,
    clippy::multiple_inherent_impl,
    clippy::multiple_unsafe_ops_per_block,
    clippy::mutex_atomic,
    clippy::mutex_integer,
    clippy::needless_collect,
    clippy::needless_pass_by_ref_mut,
    clippy::needless_raw_strings,
    clippy::non_zero_suggestions,
    clippy::nonstandard_macro_braces,
    clippy::option_if_let_else,
    clippy::or_fun_call,
    clippy::panic_in_result_fn,
    clippy::partial_pub_fields,
    clippy::pathbuf_init_then_push,
    clippy::pedantic,
    clippy::precedence_bits,
    clippy::print_stderr,
    clippy::print_stdout,
    clippy::pub_without_shorthand,
    clippy::rc_buffer,
    clippy::rc_mutex,
    clippy::read_zero_byte_vec,
    clippy::redundant_clone,
    clippy::redundant_type_annotations,
    clippy::renamed_function_params,
    clippy::ref_patterns,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::return_and_then,
    clippy::same_name_method,
    clippy::semicolon_inside_block,
    clippy::set_contains_or_insert,
    clippy::shadow_unrelated,
    clippy::significant_drop_in_scrutinee,
    clippy::significant_drop_tightening,
    clippy::str_to_string,
    clippy::string_add,
    clippy::string_lit_as_bytes,
    clippy::string_lit_chars_any,
    clippy::string_slice,
    clippy::string_to_string,
    clippy::suboptimal_flops,
    clippy::suspicious_operation_groupings,
    clippy::suspicious_xor_used_as_pow,
    clippy::tests_outside_test_module,
    clippy::todo,
    clippy::too_long_first_doc_paragraph,
    clippy::trailing_empty_array,
    clippy::transmute_undefined_repr,
    clippy::trivial_regex,
    clippy::try_err,
    clippy::undocumented_unsafe_blocks,
    clippy::unimplemented,
    clippy::uninhabited_references,
    clippy::unnecessary_safety_comment,
    clippy::unnecessary_safety_doc,
    clippy::unnecessary_self_imports,
    clippy::unnecessary_struct_initialization,
    clippy::unused_peekable,
    clippy::unused_result_ok,
    clippy::unused_trait_names,
    clippy::unwrap_in_result,
    clippy::unwrap_used,
    clippy::use_debug,
    clippy::use_self,
    clippy::useless_let_if_seq,
    clippy::verbose_file_reads,
    clippy::while_float,
    clippy::wildcard_enum_match_arm,
    ambiguous_negative_literals,
    closure_returning_async_block,
    future_incompatible,
    impl_trait_redundant_captures,
    let_underscore_drop,
    macro_use_extern_crate,
    meta_variable_misuse,
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    redundant_lifetimes,
    rust_2018_idioms,
    single_use_lifetimes,
    unit_bindings,
    unnameable_types,
    unreachable_pub,
    unstable_features,
    unused,
    variant_size_differences
)]

// TODO: Remove once tests run without complains about it.
#[cfg(test)]
mod for_docs_only {
    use proc_macro2 as _;
    use synthez as _;
}

use proc_macro::TokenStream;
use synthez_core::codegen;

/// Deriving of [`synthez::ParseAttrs`] along with a [`syn::parse::Parse`]
/// implementation to parse [`syn::Attribute`]s into a custom defined struct.
///
/// # Field requirements
///
/// Each field should be wrapped into a [`field::Container`] implementor, which
/// describes and influences the parsing logic. Use [`Required`]
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
/// use synthez::{ParseAttrs, field, parse};
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
        .unwrap_or_else(syn::Error::into_compile_error)
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
        .and_then(|i| codegen::to_tokens::derive(&i))
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
