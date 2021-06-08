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
