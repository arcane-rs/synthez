//! `#[derive(ParseAttrs)]` proc macro implementation.

use std::{collections::BTreeSet, iter};

use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    ext::IdentExt as _,
    parse::{Parse, ParseStream},
    spanned::Spanned as _,
    token,
};

use crate::{
    ext::{Data as _, Ident as _},
    parse::{
        attrs::{
            dedup,
            field::TryMerge as _,
            kind,
            validate::{rule, Validate as _},
        },
        err,
        ext::ParseBuffer as _,
    },
    ParseAttrs, Required, Spanning,
};

/// Name of the derived trait.
const TRAIT_NAME: &str = "ParseAttrs";

/// Name of the helper attribute of this `proc_macro_derive`.
const ATTR_NAME: &str = "parse";

/// Expands `#[derive(ParseAttrs)]` proc macro.
///
/// # Errors
///
/// - If the proc macro isn't applied to a struct.
/// - If parsing `#[parse]` helper attribute fails.
pub fn derive(input: syn::DeriveInput) -> syn::Result<TokenStream> {
    if !matches!(&input.data, syn::Data::Struct(_)) {
        return Err(syn::Error::new_spanned(
            input,
            format!("only structs can derive {TRAIT_NAME}"),
        ));
    }

    let out = Definition {
        ty: input.ident,
        generics: input.generics,
        fields: input
            .data
            .named_fields()?
            .into_iter()
            .map(Field::try_from)
            .collect::<syn::Result<Vec<_>>>()?,
    };

    let impl_syn_parse = out.impl_syn_parse();
    let impl_parse_attrs = out.impl_parse_attrs();
    Ok(quote! {
        #impl_syn_parse
        #impl_parse_attrs
    })
}

/// Representation of a struct implementing [`ParseAttrs`], used for code
/// generation.
#[derive(Debug)]
struct Definition {
    /// [`syn::Ident`] of this structure's type.
    ///
    /// [`syn::Ident`]: struct@syn::Ident
    ty: syn::Ident,

    /// [`syn::Generics`] of this structure's type.
    generics: syn::Generics,

    /// [`Field`]s of this structure to generate code for.
    fields: Vec<Field>,
}

impl Definition {
    /// Generates implementation of [`Parse`] trait for this struct.
    #[must_use]
    fn impl_syn_parse(&self) -> TokenStream {
        let parse_arms = self.fields.iter().map(|f| {
            let field = &f.ident;
            let ty = &f.ty;
            let kind = f.kind;
            let dedup = f.dedup;
            let arg_lits = &f.names;

            let val_ty = quote! {
                <#ty as ::synthez::field::Container<_>>::Value
            };

            let code = match kind {
                Kind::Ident => quote! {
                    <#ty as ::synthez::parse::attrs::field::TryApply<
                        _, #kind, #dedup,
                    >>::try_apply(&mut out.#field, input.parse::<#val_ty>()?)?;
                },
                Kind::Nested => quote! {
                    ::synthez::ParseBufferExt::skip_any_ident(input)?;
                    let inner;
                    let _ = ::synthez::syn::parenthesized!(inner in input);
                    <#ty as ::synthez::parse::attrs::field::TryApply<
                        _, #kind, #dedup,
                    >>::try_apply(
                        &mut out.#field,
                        ::synthez::Spanning::new(inner.parse()?, &ident),
                    )?;
                },
                Kind::Value(spaced) => {
                    let method = syn::Ident::new_on_call_site(if spaced {
                        "parse_maybe_wrapped_and_punctuated"
                    } else {
                        "parse_eq_or_wrapped_and_punctuated"
                    });

                    quote! {
                        ::synthez::ParseBufferExt::skip_any_ident(input)?;
                        for v in ::synthez::ParseBufferExt::#method::<
                            #val_ty,
                            ::synthez::syn::token::Paren,
                            ::synthez::syn::token::Comma,
                        >(input)? {
                            <#ty as ::synthez::parse::attrs::field::TryApply<
                                _, #kind, #dedup,
                            >>::try_apply(&mut out.#field, v)?;
                        }
                    }
                }
                Kind::Map => quote! {
                    ::synthez::ParseBufferExt::skip_any_ident(input)?;
                    let k = input.parse()?;
                    input.parse::<::synthez::syn::token::Eq>()?;
                    let v = input.parse()?;
                    <#ty as ::synthez::parse::attrs::field::TryApply<
                        (_, _), #kind, #dedup,
                    >>::try_apply(&mut out.#field, (k, v))?;
                },
            };

            quote! {
                #( #arg_lits )|* => { #code },
            }
        });

        let ty = &self.ty;
        let (impl_generics, ty_generics, where_clause) =
            self.generics.split_for_impl();

        quote! {
            #[automatically_derived]
            impl #impl_generics ::synthez::syn::parse::Parse
             for #ty #ty_generics
                 #where_clause
            {
                fn parse(
                    input: ::synthez::syn::parse::ParseStream<'_>,
                ) -> ::synthez::syn::Result<Self> {
                    let mut out =
                        <#ty #ty_generics as ::std::default::Default>
                            ::default();
                    while !input.is_empty() {
                        let ident =
                            ::synthez::ParseBufferExt::parse_any_ident(
                                &input.fork(),
                            )?;
                        match ident.to_string().as_str() {
                            #( #parse_arms )*
                            name => {
                                return Err(::synthez::parse::err::
                                    unknown_attr_arg(&ident, name));
                            },
                        }
                        if ::synthez::ParseBufferExt::try_parse::<
                            ::synthez::syn::token::Comma,
                        >(input)?.is_none() && !input.is_empty() {
                            return Err(::synthez::parse::err::
                                expected_followed_by_comma(&ident));
                        }
                    }
                    Ok(out)
                }
            }
        }
    }

    /// Generates implementation of [`ParseAttrs`] trait for this struct.
    #[must_use]
    fn impl_parse_attrs(&self) -> TokenStream {
        let ty = &self.ty;
        let (impl_generics, ty_generics, where_clause) =
            self.generics.split_for_impl();

        let try_merge_fields = self.fields.iter().map(Field::gen_merge);

        let validate_provided_fields =
            self.fields.iter().map(Field::gen_validate_provided);
        let validate_nested_fields =
            self.fields.iter().filter_map(Field::gen_validate_nested);
        let validate_custom_fields = self.fields.iter().flat_map(|f| {
            let field = &f.ident;
            f.validators.iter().map(move |validator| {
                quote! {
                    #validator(&self.#field)?;
                }
            })
        });

        let fallback_nested_fields =
            self.fields.iter().filter_map(Field::gen_fallback_nested);
        let fallback_custom_fields = self.fields.iter().flat_map(|f| {
            let field = &f.ident;
            f.fallbacks.iter().map(move |fallback| {
                quote! {
                    #fallback(&mut self.#field, attrs)?;
                }
            })
        });

        quote! {
            #[automatically_derived]
            impl #impl_generics ::synthez::parse::Attrs for #ty #ty_generics
                 #where_clause
            {
                fn try_merge(
                    mut self,
                    another: Self,
                ) -> ::synthez::syn::Result<Self> {
                    #( #try_merge_fields )*
                    Ok(self)
                }

                fn validate(
                    &self,
                    attr_name: &str,
                    item_span: ::synthez::proc_macro2::Span,
                ) -> ::synthez::syn::Result<()> {
                    #( #validate_provided_fields )*
                    #( #validate_nested_fields )*
                    #( #validate_custom_fields )*
                    Ok(())
                }

                fn fallback(
                    &mut self,
                    attrs: &[::synthez::syn::Attribute],
                ) -> ::synthez::syn::Result<()> {
                    #( #fallback_nested_fields )*
                    #( #fallback_custom_fields )*
                    Ok(())
                }
            }
        }
    }
}

/// Representation of a [`ParseAttrs`]'s field, used for code generation.
#[derive(Debug)]
struct Field {
    /// [`syn::Ident`] of this [`Field`] in the original code (without possible
    /// `r#` part).
    ///
    /// [`syn::Ident`]: struct@syn::Ident
    ident: syn::Ident,

    /// [`syn::Type`] of this [`Field`] (with [`field::Container`]).
    ///
    /// [`field::Container`]: crate::field::Container
    ty: syn::Type,

    /// Parsing [`kind`] to use for this [`Field`] in the generated code.
    kind: Kind,

    /// [`dedup`]lication strategy to use for this [`Field`] in the generated
    /// code.
    dedup: Dedup,

    /// Names [`syn::Attribute`]'s arguments to parse this [`Field`] from in the
    /// generated code.
    names: Vec<String>,

    /// Additional custom validators to apply to this [`Field`] in the generated
    /// code.
    validators: Vec<syn::Expr>,

    /// Additional custom fallback functions to apply to this [`Field`] in the
    /// generated code.
    fallbacks: Vec<syn::Expr>,
}

impl TryFrom<syn::Field> for Field {
    type Error = syn::Error;

    fn try_from(field: syn::Field) -> syn::Result<Self> {
        let attrs = FieldAttrs::parse_attrs(ATTR_NAME, &field)?;

        let field_span = field.span();
        let ident = field.ident.ok_or_else(move || {
            syn::Error::new(field_span, "Named field expected")
        })?;

        let mut names = if attrs.args.is_empty() {
            iter::once(ident.unraw()).collect()
        } else {
            attrs.args
        };
        names.try_merge_self::<kind::Value, dedup::Unique>(attrs.aliases)?;

        Ok(Self {
            ident,
            ty: field.ty,
            kind: **attrs.kind,
            dedup: attrs.dedup.as_deref().copied().unwrap_or_default(),
            names: names.into_iter().map(|n| n.to_string()).collect(),
            validators: attrs.validators,
            fallbacks: attrs.fallbacks,
        })
    }
}

impl Field {
    /// Generates code of merging this [`Field`] with another one.
    #[must_use]
    fn gen_merge(&self) -> TokenStream {
        let field = &self.ident;
        let ty = &self.ty;
        let kind = self.kind;
        let dedup = self.dedup;

        quote! {
            <#ty as ::synthez::parse::attrs::field::TryApplySelf<
                _, #kind, #dedup,
            >>::try_apply_self(&mut self.#field, another.#field)?;
        }
    }

    /// Generates code of [`rule::Provided`] validation for this [`Field`].
    #[must_use]
    fn gen_validate_provided(&self) -> TokenStream {
        let field = &self.ident;
        let ty = &self.ty;

        let names_len = self.names.len();
        let arg_names = if names_len > 1 {
            format!(
                "either `{}` or `{}`",
                self.names[..(names_len - 1)].join("`, `"),
                self.names[names_len - 1],
            )
        } else {
            format!("`{}`", self.names[0])
        };
        let err_msg =
            format!("{arg_names} argument of `#[{{}}]` attribute {{}}");

        quote! {
            if let Err(e) = <#ty as ::synthez::parse::attrs::Validation<
                ::synthez::parse::attrs::validate::rule::Provided,
            >>::validation(&self.#field) {
                return Err(::synthez::syn::Error::new(
                    item_span,
                    format!(#err_msg, attr_name, e),
                ));
            }
        }
    }

    /// Generates code of [`kind::Nested`] validation for this [`Field`], if it
    /// represents the one.
    #[must_use]
    fn gen_validate_nested(&self) -> Option<TokenStream> {
        if self.kind != Kind::Nested {
            return None;
        }

        let field = &self.ident;
        let attr_fmt = format!("{{}}({})", self.names[0]);

        Some(quote! {
            for v in &self.#field {
                ::synthez::parse::Attrs::validate(
                    &**v,
                    &format!(#attr_fmt, attr_name),
                    ::synthez::spanned::IntoSpan::into_span(v),
                )?;
            }
        })
    }

    /// Generates code of [`kind::Nested`] fallback for this [`Field`], if it
    /// represents the one.
    #[must_use]
    fn gen_fallback_nested(&self) -> Option<TokenStream> {
        if self.kind != Kind::Nested {
            return None;
        }

        let field = &self.ident;
        let ty = &self.ty;

        Some(quote! {
            if !<#ty as ::synthez::field::Container<_>>::is_empty(
                &self.#field,
            ) {
                for v in &mut self.#field {
                    ::synthez::parse::Attrs::fallback(&mut **v, attrs)?;
                }
            }
        })
    }
}

/// Representation of a `#[parse]` attribute used along with a
/// `#[derive(ParseAttrs)]` proc macro and placed on struct fields.
#[derive(Debug, Default)]
struct FieldAttrs {
    /// [`kind`] of the [`ParseAttrs`]'s field parsing.
    // #[parse(ident, args(ident, nested, value, map))]
    kind: Required<Spanning<Kind>>,

    /// Names of [`syn::Attribute`]'s arguments to use for parsing __instead
    /// of__ the [`ParseAttrs`]'s field's [`syn::Ident`].
    ///
    /// [`syn::Ident`]: struct@syn::Ident
    // #[parse(value, alias = arg)]
    args: BTreeSet<syn::Ident>,

    /// Names of [`syn::Attribute`]'s arguments to use for parsing __along
    /// with__ the [`ParseAttrs`]'s field's [`syn::Ident`].
    ///
    /// [`syn::Ident`]: struct@syn::Ident
    // #[parse(value, alias = alias)]
    aliases: BTreeSet<syn::Ident>,

    /// [`dedup`]lication strategy of how multiple values of the
    /// [`ParseAttrs`]'s field should be merged.
    ///
    /// Default is [`Dedup::Unique`].
    // #[parse(value)]
    dedup: Option<Spanning<Dedup>>,

    /// Additional custom validators to use for the [`ParseAttrs`]'s field.
    // #[parse(value, arg = validate)]
    validators: Vec<syn::Expr>,

    /// Additional custom fallback functions to use for the [`ParseAttrs`]'s
    /// field.
    // #[parse(value, alias = fallback)]
    fallbacks: Vec<syn::Expr>,
}

impl Parse for FieldAttrs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut out = Self::default();
        while !input.is_empty() {
            let ident = input.fork().parse_any_ident()?;
            match ident.to_string().as_str() {
                "ident" | "nested" | "value" | "map" => {
                    out.kind.try_merge::<kind::Ident, dedup::Unique>(
                        input.parse::<Spanning<Kind>>()?,
                    )?;
                }
                "arg" | "args" => {
                    input.skip_any_ident()?;
                    for val in input.parse_eq_or_wrapped_and_punctuated::<
                        syn::Ident, token::Paren, token::Comma,
                    >()? {
                        out.args.try_merge::<kind::Value, dedup::Unique>(val)?;
                    }
                }
                "alias" | "aliases" => {
                    input.skip_any_ident()?;
                    for v in input.parse_eq_or_wrapped_and_punctuated::<
                        syn::Ident, token::Paren, token::Comma,
                    >()? {
                        out.aliases.try_merge::<kind::Value, dedup::Unique>(v)?;
                    }
                }
                "dedup" => {
                    input.skip_any_ident()?;
                    for val in input.parse_eq_or_wrapped_and_punctuated::<
                        Spanning<Dedup>, token::Paren, token::Comma,
                    >()? {
                        out.dedup.try_merge::<kind::Value, dedup::Unique>(val)?;
                    }
                }
                "validate" => {
                    input.skip_any_ident()?;
                    for v in input.parse_eq_or_wrapped_and_punctuated::<
                        syn::Expr, token::Paren, token::Comma,
                    >()? {
                        out.validators.try_merge::<
                            kind::Value, dedup::Unique,
                        >(v)?;
                    }
                }
                "fallbacks" | "fallback" => {
                    input.skip_any_ident()?;
                    for v in input.parse_eq_or_wrapped_and_punctuated::<
                        syn::Expr, token::Paren, token::Comma,
                    >()? {
                        out.fallbacks.try_merge::<
                            kind::Value, dedup::Unique,
                        >(v)?;
                    }
                }
                name => {
                    return Err(err::unknown_attr_arg(&ident, name));
                }
            }
            if input.try_parse::<token::Comma>()?.is_none() && !input.is_empty()
            {
                return Err(err::expected_followed_by_comma(&ident));
            }
        }
        Ok(out)
    }
}

impl ParseAttrs for FieldAttrs {
    fn try_merge(mut self, another: Self) -> syn::Result<Self> {
        self.kind.try_merge_self::<kind::Value, dedup::Unique>(another.kind)?;
        self.args.try_merge_self::<kind::Value, dedup::Unique>(another.args)?;
        self.aliases
            .try_merge_self::<kind::Value, dedup::Unique>(another.aliases)?;
        self.dedup
            .try_merge_self::<kind::Value, dedup::Unique>(another.dedup)?;
        self.validators
            .try_merge_self::<kind::Value, dedup::Unique>(another.validators)?;
        self.fallbacks
            .try_merge_self::<kind::Value, dedup::Unique>(another.fallbacks)?;
        Ok(self)
    }

    fn validate(&self, attr_name: &str, item_span: Span) -> syn::Result<()> {
        if self.kind.validate::<rule::Provided>().is_err() {
            return Err(syn::Error::new(
                item_span,
                format!(
                    "either `ident`, `value` or `map` argument of \
                     `#[{attr_name}]` attribute is expected",
                ),
            ));
        }
        Ok(())
    }
}

/// Field [`kind`] of parsing it from [`syn::Attribute`]s.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Kind {
    /// Field is parsed as a simple [`syn::Ident`].
    ///
    /// [`syn::Ident`]: struct@syn::Ident
    Ident,

    /// Field is parsed as a nested structure implementing [`ParseAttrs`].
    Nested,

    /// Field is parsed as values behind a [`syn::Ident`].
    ///
    /// Boolean refers to whether the value and the [`syn::Ident`] are separated
    /// with spaces only.
    ///
    /// [`syn::Ident`]: struct@syn::Ident
    Value(bool),

    /// Field is parsed as as key-value pairs behind a [`syn::Ident`].
    ///
    /// [`syn::Ident`]: struct@syn::Ident
    Map,
}

impl Parse for Spanning<Kind> {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let ident = input.parse::<syn::Ident>()?;
        Ok(Self::new(
            match ident.to_string().as_str() {
                "ident" => Kind::Ident,
                "nested" => Kind::Nested,
                "value" => {
                    if input.is_next::<token::Paren>() {
                        let inner;
                        _ = syn::parenthesized!(inner in input);
                        let inner = inner.parse::<syn::Ident>()?;
                        let val = inner.to_string();
                        if val != "spaced" {
                            return Err(syn::Error::new_spanned(
                                inner,
                                format!("invalid value setting: {val} "),
                            ));
                        }
                        Kind::Value(true)
                    } else {
                        Kind::Value(false)
                    }
                }
                "map" => Kind::Map,
                val => {
                    return Err(syn::Error::new_spanned(
                        ident,
                        format!("invalid kind value: {val} "),
                    ));
                }
            },
            &ident,
        ))
    }
}

impl ToTokens for Kind {
    fn to_tokens(&self, out: &mut TokenStream) {
        let variant = syn::Ident::new_on_call_site(match self {
            Self::Ident => "Ident",
            Self::Nested => "Nested",
            Self::Value(_) => "Value",
            Self::Map => "Map",
        });
        (quote! {
            ::synthez::parse::attrs::kind::#variant
        })
        .to_tokens(out);
    }
}

/// Field [`dedup`]lication strategy parsed from [`syn::Attribute`]s.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum Dedup {
    /// Only a single value of the field is allowed to appear.
    #[default]
    Unique,

    /// Only the first parsed value of the field is picked.
    First,

    /// Only the last parsed value of the field is picked.
    Last,
}

impl Parse for Spanning<Dedup> {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let ident = input.parse::<syn::Ident>()?;
        Ok(Self::new(
            match ident.to_string().as_str() {
                "unique" => Dedup::Unique,
                "first" => Dedup::First,
                "last" => Dedup::Last,
                val => {
                    return Err(syn::Error::new_spanned(
                        ident,
                        format!("invalid dedup value: {val} "),
                    ));
                }
            },
            &ident,
        ))
    }
}

impl ToTokens for Dedup {
    fn to_tokens(&self, out: &mut TokenStream) {
        let variant = syn::Ident::new_on_call_site(match self {
            Self::Unique => "Unique",
            Self::First => "First",
            Self::Last => "Last",
        });
        (quote! {
            ::synthez::parse::attrs::dedup::#variant
        })
        .to_tokens(out);
    }
}
