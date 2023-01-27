synthez
=======

[![crates.io](https://img.shields.io/crates/v/synthez.svg "crates.io")](https://crates.io/crates/synthez)
[![Rust 1.62+](https://img.shields.io/badge/rustc-1.62+-lightgray.svg "Rust 1.62+")](https://blog.rust-lang.org/2022/06/30/Rust-1.62.0.html)
[![Unsafe Forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg "Unsafe forbidden")](https://github.com/rust-secure-code/safety-dance)  
[![CI](https://github.com/arcane-rs/synthez/workflows/CI/badge.svg?branch=main "CI")](https://github.com/arcane-rs/synthez/actions?query=workflow%3ACI+branch%3Amain)
[![Rust docs](https://docs.rs/synthez/badge.svg "Rust docs")](https://docs.rs/synthez)

[API Docs](https://docs.rs/synthez) |
[Changelog](https://github.com/arcane-rs/synthez/blob/main/CHANGELOG.md)

Steroids for [`syn`], [`quote`] and [`proc_macro2`] crates.




## Cargo features


### `full`

Same as `full` feature of [`syn`] crate.

Enables support of data structures for representing the syntax tree of all valid Rust source code, including items and expressions.




## Example of writing `proc_macro_derive`

This is an example of how this library can be used to write a simplified [`proc_macro_derive`] for deriving a [`From`] implementations.

```rust
# use std::collections::HashMap;
#
# use synthez::{proc_macro2::{Span, TokenStream}, quote::quote, syn};
use synthez::{DataExt as _, ParseAttrs, ToTokens};

pub fn derive(input: syn::DeriveInput) -> syn::Result<TokenStream> {
    let attrs = Attrs::parse_attrs("from", &input)?;
    match (attrs.forward.is_some(), !attrs.custom.is_empty()) {
        (true, true) => Err(syn::Error::new_spanned(
            input,
            "`forward` and `on` arguments are mutually exclusive",
        )),
        (false, false) => Err(syn::Error::new_spanned(
            input,
            "either `forward` or `on` argument is expected",
        )),

        // #[from(forward)]
        (true, _) => {
            if !matches!(&input.data, syn::Data::Struct(_)) {
                return Err(syn::Error::new_spanned(
                    input,
                    "only tuple structs can forward-derive From",
                ));
            }
            let fields = input.data.unnamed_fields()?;
            if fields.len() > 1 {
                return Err(syn::Error::new_spanned(
                    fields,
                    "only single-field tuple structs can forward-derive \
                     From",
                ));
            }
            let definition = ForwardDefinition {
                ty: input.ident,
                inner_ty: fields.into_iter().last().unwrap().ty,
            };
            Ok(quote! {
                #definition
            })
        }

        // #[from(on <type> = <func>)]
        (_, true) => {
            let definitions =
                CustomDefinitions { ty: input.ident, funcs: attrs.custom };
            Ok(quote! {
                #definitions
            })
        }
    }
}

#[derive(Default, ParseAttrs)]
struct Attrs {
    #[parse(ident)]
    forward: Option<syn::Ident>,
    #[parse(map, arg = on)]
    custom: HashMap<syn::Type, syn::Expr>,
}

#[derive(ToTokens)]
#[to_tokens(append(impl_from))]
struct ForwardDefinition {
    ty: syn::Ident,
    inner_ty: syn::Type,
}

impl ForwardDefinition {
    fn impl_from(&self) -> TokenStream {
        let (ty, inner_ty) = (&self.ty, &self.inner_ty);
        quote! {
            impl<T> From<T> for #ty where #inner_ty: From<T> {
                fn from(v: T) -> Self {
                    Self(v.into())
                }
            }
        }
    }
}

#[derive(ToTokens)]
#[to_tokens(append(impl_froms))]
struct CustomDefinitions {
    ty: syn::Ident,
    funcs: HashMap<syn::Type, syn::Expr>,
}

impl CustomDefinitions {
    fn impl_froms(&self) -> TokenStream {
        let ty = &self.ty;
        // We sort here for tests below not failing due to undetermined
        // order only. Real implementation may omit this.
        let mut sorted = self.funcs.iter().collect::<Vec<_>>();
        sorted.sort_unstable_by(|(ty1, _), (ty2, _)| {
            quote!(#ty1).to_string().cmp(&quote!(#ty2).to_string())
        });
        let impls = sorted.into_iter().map(move |(from_ty, func)| {
            quote! {
                impl From<#from_ty> for #ty {
                    fn from(v: #from_ty) -> Self {
                        #func(v)
                    }
                }
            }
        });
        quote! { #( #impls )* }
    }
}

# fn main() {
let input = syn::parse_quote! {
    #[derive(From)]
    #[from(forward)]
    struct Id(u64);
};
let output = quote! {
    impl<T> From<T> for Id where u64: From<T> {
        fn from(v: T) -> Self {
            Self(v.into())
        }
    }
};
assert_eq!(derive(input).unwrap().to_string(), output.to_string());

let input = syn::parse_quote! {
    #[derive(From)]
    #[from(on bool = Self::parse_bool)]
    #[from(on u8 = from_u8_to_maybe)]
    enum Maybe {
        Yes,
        No,
    }
};
let output = quote! {
    impl From<bool> for Maybe {
        fn from(v: bool) -> Self {
            Self::parse_bool(v)
        }
    }
    impl From<u8> for Maybe {
        fn from(v: u8) -> Self {
            from_u8_to_maybe(v)
        }
    }
};
assert_eq!(derive(input).unwrap().to_string(), output.to_string());
# }
```




## License

This software is subject to the terms of the [Blue Oak Model License 1.0.0](https://github.com/instrumentisto/tracerr-rs/blob/main/LICENSE.md). If a copy of the [BlueOak-1.0.0](https://spdx.org/licenses/BlueOak-1.0.0.html) license was not distributed with this file, You can obtain one at <https://blueoakcouncil.org/license/1.0.0>.




[`From`]: https://doc.rust-lang.org/stable/std/convert/trait.From.html
[`proc_macro2`]: https://docs.rs/proc_macro2
[`proc_macro_derive`]: https://doc.rust-lang.org/reference/procedural-macros.html#derive-macros
[`quote`]: https://docs.rs/quote
[`syn`]: https://docs.rs/syn
