use synthez::{proc_macro2::TokenStream, quote::quote, ToTokens};

#[derive(ToTokens)]
#[to_tokens(append(impl_tokens))]
struct Some;

impl Some {
    fn impl_tokens(&self) -> TokenStream {
        quote! {
            wut
        }
    }
}
