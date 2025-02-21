#![forbid(non_ascii_idents, unsafe_code)]

use synthez::{ToTokens, proc_macro2::TokenStream, quote::quote};

#[derive(ToTokens)]
#[to_tokens(append(impl_tokens))]
#[to_tokens(append(more_tokens))]
struct Some;

impl Some {
    fn impl_tokens(&self) -> TokenStream {
        quote! {
            whoopsie
        }
    }

    fn more_tokens(&self) -> TokenStream {
        quote! {
            daisy
        }
    }
}

#[test]
fn appends_tokens() {
    let some = Some;
    let code = quote! { #some };

    assert_eq!(code.to_string(), "whoopsie daisy");
}
