use std::collections::HashMap;

use synthez::{field, parse, ParseAttrs, Required, Spanning};

#[derive(Default, ParseAttrs)]
struct Sub {
    #[parse(ident, arg = val, validate(is_correct, is_correct2))]
    value: Required<syn::Ident>,
}

#[derive(Default, ParseAttrs)]
struct MyAttrs {
    #[parse(value, alias = named, fallback = field::if_empty(parse::attr::doc))]
    name: Option<syn::LitStr>,
    #[parse(nested)]
    nested: Required<Spanning<Sub>>,
    #[parse(map, arg = on)]
    multi: HashMap<syn::Type, syn::Expr>,
}

fn is_correct(_: &syn::Ident) -> syn::Result<()> {
    Ok(())
}

fn is_correct2(_: &syn::Ident) -> syn::Result<()> {
    Ok(())
}
