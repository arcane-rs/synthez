use std::{
    collections::HashSet,
    convert::TryFrom,
    iter::{self, FromIterator as _},
};

use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens, TokenStreamExt as _};
use syn::{
    parse::{Parse, ParseStream},
    spanned::Spanned as _,
    token,
};

use crate::{
    ext::{DataExt as _, IdentExt as _},
    parse::{
        attrs::{
            dedup,
            field::TryMerge as _,
            kind,
            validate::{rule, IsValid as _},
        },
        err, ParseBufferExt as _,
    },
    ParseAttrs, Required, Spanning,
};

/// Name of the derived trait.
const TRAIT_NAME: &str = "ParseAttrs";

/// Name of the helper attribute of this `proc_macro_derive`.
const ATTR_NAME: &str = "parse";

pub fn derive(input: syn::DeriveInput) -> syn::Result<TokenStream> {
    if !matches!(&input.data, syn::Data::Struct(_)) {
        return Err(syn::Error::new(
            input.span(),
            format!("Only structs can derive {}", TRAIT_NAME),
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

#[derive(Debug)]
struct Definition {
    ty: syn::Ident,
    generics: syn::Generics,
    fields: Vec<Field>,
}

impl Definition {
    fn impl_syn_parse(&self) -> TokenStream {
        let ty = &self.ty;
        let (impl_generics, ty_generics, where_clause) =
            self.generics.split_for_impl();

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
                    ::synthez::parse::ParseBufferExt::skip_any_ident(input)?;
                    let inner;
                    let _ = ::synthez::syn::parenthesized!(inner in input);
                    <#ty as ::synthez::parse::attrs::field::TryApply<
                        _, #kind, #dedup,
                    >>::try_apply(&mut out.#field, inner.parse::<#val_ty>()?)?;
                },
                Kind::Value(spaced) => {
                    let method = syn::Ident::new_on_call_site(if spaced {
                        "parse_maybe_wrapped_and_punctuated"
                    } else {
                        "parse_eq_or_wrapped_and_punctuated"
                    });

                    quote! {
                        ::synthez::parse::ParseBufferExt::skip_any_ident(
                            input,
                        )?;
                        for v in ::synthez::parse::ParseBufferExt::#method::<
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
                    ::synthez::parse::ParseBufferExt::skip_any_ident(input)?;
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

        quote! {
            #[automatically_derived]
            impl#impl_generics ::synthez::syn::parse::Parse for #ty#ty_generics
                #where_clause
            {
                fn parse(
                    input: ::synthez::syn::parse::ParseStream<'_>,
                ) -> ::synthez::syn::Result<Self> {
                    let mut out = <#ty#ty_generics as Default>::default();
                    while !input.is_empty() {
                        let ident =
                            ::synthez::parse::ParseBufferExt::parse_any_ident(
                                &input.fork(),
                            )?;
                        match ident.to_string().as_str() {
                            #( #parse_arms )*
                            name => {
                                return Err(::synthez::parse::err::
                                    unknown_attr_arg(&ident, name));
                            },
                        }
                        if ::synthez::parse::ParseBufferExt::try_parse::<
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

    fn impl_parse_attrs(&self) -> TokenStream {
        let ty = &self.ty;
        let (impl_generics, ty_generics, where_clause) =
            self.generics.split_for_impl();

        let try_merge_fields = self.fields.iter().map(|f| {
            let field = &f.ident;
            let ty = &f.ty;
            let kind = f.kind;
            let dedup = f.dedup;

            quote! {
                <#ty as ::synthez::parse::attrs::field::TryApplySelf<
                    _, #kind, #dedup,
                >>::try_apply_self(&mut self.#field, another.#field)?;
            }
        });

        let validate_inited_fields = self.fields.iter().map(|f| {
            let field = &f.ident;
            let ty = &f.ty;

            let arg_names = if f.names.len() > 1 {
                format!(
                    "either `{}` or `{}`",
                    &f.names[..(f.names.len() - 1)].join("`, `"),
                    f.names.last().unwrap(),
                )
            } else {
                format!("`{}`", f.names.first().unwrap())
            };
            let err_msg = format!(
                "{} argument of `#[{{}}]` attribute is expected",
                arg_names,
            );

            quote! {
                if !<#ty as ::synthez::parse::attrs::Validate<
                    ::synthez::parse::attrs::validate::rule::Provided,
                >>::validate(&self.#field) {
                    return Err(::synthez::syn::Error::new(
                        item_span,
                        format!(#err_msg, attr_name),
                    ));
                }
            }
        });

        let validate_custom_fields = self
            .fields
            .iter()
            .map(|f| {
                let field = &f.ident;
                f.validators.iter().map(move |validator| {
                    quote! {
                        #validator(&self.#field)?;
                    }
                })
            })
            .flatten();

        let fallback_fields = self
            .fields
            .iter()
            .map(|f| {
                let field = &f.ident;
                f.fallbacks.iter().map(move |fallback| {
                    quote! {
                        #fallback(&mut self.#field, attrs)?;
                    }
                })
            })
            .flatten()
            .collect::<Vec<_>>();
        let fallback_method = (!fallback_fields.is_empty()).then(|| {
            quote! {
                fn fallback(
                    &mut self,
                    attrs: &[::synthez::syn::Attribute],
                ) -> ::synthez::syn::Result<()> {
                    #( #fallback_fields )*
                    Ok(())
                }
            }
        });

        quote! {
            #[automatically_derived]
            impl#impl_generics ::synthez::parse::Attrs for #ty#ty_generics
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
                    #( #validate_inited_fields )*
                    #( #validate_custom_fields )*
                    Ok(())
                }

                #fallback_method
            }
        }
    }
}

#[derive(Debug)]
struct Field {
    ident: syn::Ident,
    ty: syn::Type,
    kind: Kind,
    dedup: Dedup,
    names: Vec<String>,
    validators: Vec<syn::Expr>,
    fallbacks: Vec<syn::Expr>,
}

impl TryFrom<syn::Field> for Field {
    type Error = syn::Error;

    fn try_from(field: syn::Field) -> syn::Result<Self> {
        let attrs = FieldAttrs::parse_attrs(ATTR_NAME, &field)?;
        let ident = field.ident.unwrap();

        let mut names = if attrs.args.is_empty() {
            HashSet::from_iter(iter::once(ident.clone()))
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

#[derive(Debug, Default)]
struct FieldAttrs {
    // #[parse(ident, args(ident, value, map))]
    kind: Required<Spanning<Kind>>,

    // #[parse(value, alias = arg)]
    args: HashSet<syn::Ident>,

    // #[parse(value, alias = alias)]
    aliases: HashSet<syn::Ident>,

    // #[parse(value)]
    dedup: Option<Spanning<Dedup>>,

    // #[parse(value, arg = validate)]
    validators: Vec<syn::Expr>,

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
        if !self.kind.is_valid::<rule::Provided>() {
            return Err(syn::Error::new(
                item_span,
                format!(
                    "either `ident`, `value` or `map` argument of `#[{}]` \
                     attribute is expected",
                    attr_name,
                ),
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Kind {
    Ident,
    Nested,
    Value(bool),
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
                        let _ = syn::parenthesized!(inner in input);
                        let inner = inner.parse::<syn::Ident>()?;
                        let val = inner.to_string();
                        if val != "spaced" {
                            return Err(syn::Error::new(
                                inner.span(),
                                format!("invalid value setting: {} ", val),
                            ));
                        }
                        Kind::Value(true)
                    } else {
                        Kind::Value(false)
                    }
                }
                "map" => Kind::Map,
                val => {
                    return Err(syn::Error::new(
                        ident.span(),
                        format!("invalid kind value: {} ", val),
                    ))
                }
            },
            &ident,
        ))
    }
}

impl ToTokens for Kind {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let variant = syn::Ident::new_on_call_site(match self {
            Self::Ident => "Ident",
            Self::Nested => "Nested",
            Self::Value(_) => "Value",
            Self::Map => "Map",
        });
        tokens.append_all(&[quote! {
            ::synthez::parse::attrs::kind::#variant
        }])
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Dedup {
    Unique,
    First,
    Last,
}

impl Default for Dedup {
    #[inline]
    fn default() -> Self {
        Self::Unique
    }
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
                    return Err(syn::Error::new(
                        ident.span(),
                        format!("invalid dedup value: {} ", val),
                    ))
                }
            },
            &ident,
        ))
    }
}

impl ToTokens for Dedup {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let variant = syn::Ident::new_on_call_site(match self {
            Self::Unique => "Unique",
            Self::First => "First",
            Self::Last => "Last",
        });
        tokens.append_all(&[quote! {
            ::synthez::parse::attrs::dedup::#variant
        }])
    }
}
