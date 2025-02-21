#![forbid(non_ascii_idents, unsafe_code)]

mod ident {
    use synthez::{IdentExt as _, ParseAttrs, syn};

    mod option {
        use super::*;

        #[derive(Debug, Default, ParseAttrs)]
        struct Attr {
            #[parse(ident)]
            ignore: Option<syn::Ident>,
        }

        #[test]
        fn allows_present() {
            let input: syn::DeriveInput = syn::parse_quote! {
                #[attr(ignore)]
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_ok(), "failed: {}", res.unwrap_err());

            assert_eq!(
                res.unwrap().ignore,
                Some(syn::Ident::new_on_call_site("ignore")),
            );
        }

        #[test]
        fn allows_absent() {
            let input: syn::DeriveInput = syn::parse_quote! {
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_ok(), "failed: {}", res.unwrap_err());

            assert_eq!(res.unwrap().ignore, None);
        }
    }

    mod option_fallback {
        use synthez::field;

        use super::*;

        #[derive(Debug, Default, ParseAttrs)]
        struct Attr {
            #[parse(ident, fallback = field::if_empty(use_back))]
            ignore: Option<syn::Ident>,
        }

        fn use_back(_: &[syn::Attribute]) -> syn::Result<Option<syn::Ident>> {
            Ok(Some(syn::Ident::new_on_call_site("fallen")))
        }

        #[test]
        fn omits_not_empty() {
            let input: syn::DeriveInput = syn::parse_quote! {
                #[attr(ignore)]
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_ok(), "failed: {}", res.unwrap_err());

            assert_eq!(
                res.unwrap().ignore,
                Some(syn::Ident::new_on_call_site("ignore")),
            );
        }

        #[test]
        fn uses_if_empty() {
            let input: syn::DeriveInput = syn::parse_quote! {
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_ok(), "failed: {}", res.unwrap_err());

            assert_eq!(
                res.unwrap().ignore,
                Some(syn::Ident::new_on_call_site("fallen")),
            );
        }
    }

    mod required {
        use synthez::Required;

        use super::*;

        #[derive(Debug, Default, ParseAttrs)]
        struct Attr {
            #[parse(ident)]
            required: Required<syn::Ident>,
        }

        #[test]
        fn allows_present() {
            let input: syn::DeriveInput = syn::parse_quote! {
                #[attr(required)]
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_ok(), "failed: {}", res.unwrap_err());

            assert_eq!(
                *res.unwrap().required,
                syn::Ident::new_on_call_site("required"),
            );
        }

        #[test]
        fn forbids_absent() {
            let input: syn::DeriveInput = syn::parse_quote! {
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_err(), "should fail, but is ok");

            let err = res.unwrap_err().to_string();
            assert!(
                err.contains(
                    "`required` argument of `#[attr]` attribute is expected",
                ),
                "wrong err:\n{err}",
            );
        }
    }

    mod required_fallback {
        use synthez::{Required, field};

        use super::*;

        #[derive(Debug, Default, ParseAttrs)]
        struct Attr {
            #[parse(ident, fallback = field::if_empty(use_back))]
            required: Required<syn::Ident>,
        }

        fn use_back(_: &[syn::Attribute]) -> syn::Result<Option<syn::Ident>> {
            Ok(Some(syn::Ident::new_on_call_site("fallen")))
        }

        #[test]
        fn omits_not_empty() {
            let input: syn::DeriveInput = syn::parse_quote! {
                #[attr(required)]
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_ok(), "failed: {}", res.unwrap_err());

            assert_eq!(
                *res.unwrap().required,
                syn::Ident::new_on_call_site("required"),
            );
        }

        #[test]
        fn uses_if_empty() {
            let input: syn::DeriveInput = syn::parse_quote! {
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_ok(), "failed: {}", res.unwrap_err());

            assert_eq!(
                *res.unwrap().required,
                syn::Ident::new_on_call_site("fallen"),
            );
        }
    }

    mod renamed {
        use super::*;

        #[derive(Debug, Default, ParseAttrs)]
        struct Attr {
            #[parse(ident, arg = skip)]
            ignore: Option<syn::Ident>,
        }

        #[test]
        fn accepts_new_arg_name() {
            let input: syn::DeriveInput = syn::parse_quote! {
                #[attr(skip)]
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_ok(), "failed: {}", res.unwrap_err());

            assert_eq!(
                res.unwrap().ignore,
                Some(syn::Ident::new_on_call_site("skip")),
            );
        }

        #[test]
        fn errs_on_old_arg_name() {
            let input: syn::DeriveInput = syn::parse_quote! {
                #[attr(ignore)]
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_err(), "should fail, but ok");

            let err = res.unwrap_err().to_string();
            assert_eq!(err, "unknown `ignore` attribute argument");
        }
    }

    mod aliased {
        use super::*;

        #[derive(Debug, Default, ParseAttrs)]
        struct Attr {
            #[parse(ident, alias = skip)]
            ignore: Option<syn::Ident>,
        }

        #[test]
        fn accepts_alias_arg_name() {
            let input: syn::DeriveInput = syn::parse_quote! {
                #[attr(skip)]
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_ok(), "failed: {}", res.unwrap_err());

            assert_eq!(
                res.unwrap().ignore,
                Some(syn::Ident::new_on_call_site("skip")),
            );
        }

        #[test]
        fn accepts_old_arg_name() {
            let input: syn::DeriveInput = syn::parse_quote! {
                #[attr(ignore)]
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_ok(), "failed: {}", res.unwrap_err());

            assert_eq!(
                res.unwrap().ignore,
                Some(syn::Ident::new_on_call_site("ignore")),
            );
        }
    }

    mod renamed_and_aliased {
        use super::*;

        #[derive(Debug, Default, ParseAttrs)]
        struct Attr {
            #[parse(ident, arg = skip, alias = ignored)]
            ignore: Option<syn::Ident>,
        }

        #[test]
        fn accepts_new_arg_name() {
            let input: syn::DeriveInput = syn::parse_quote! {
                #[attr(skip)]
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_ok(), "failed: {}", res.unwrap_err());

            assert_eq!(
                res.unwrap().ignore,
                Some(syn::Ident::new_on_call_site("skip")),
            );
        }

        #[test]
        fn accepts_alias_arg_name() {
            let input: syn::DeriveInput = syn::parse_quote! {
                #[attr(ignored)]
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_ok(), "failed: {}", res.unwrap_err());

            assert_eq!(
                res.unwrap().ignore,
                Some(syn::Ident::new_on_call_site("ignored")),
            );
        }

        #[test]
        fn errs_on_old_arg_name() {
            let input: syn::DeriveInput = syn::parse_quote! {
                #[attr(ignore)]
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_err(), "should fail, but ok");

            let err = res.unwrap_err().to_string();
            assert_eq!(err, "unknown `ignore` attribute argument");
        }
    }

    mod dedup_default {
        use super::*;

        #[derive(Debug, Default, ParseAttrs)]
        struct Attr {
            #[parse(ident, dedup = unique, alias = skip)]
            ignore: Option<syn::Ident>,
        }

        #[test]
        fn is_unique() {
            let input: syn::DeriveInput = syn::parse_quote! {
                #[attr(ignore, skip)]
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_err(), "should fail, but ok");

            let err = res.unwrap_err().to_string();
            assert_eq!(err, "duplicated attribute's argument found");
        }
    }

    mod dedup_unique {
        use super::*;

        #[derive(Debug, Default, ParseAttrs)]
        struct Attr {
            #[parse(ident, dedup = unique, alias = skip)]
            ignore: Option<syn::Ident>,
        }

        #[test]
        fn forbids_repeated_arg() {
            let input: syn::DeriveInput = syn::parse_quote! {
                #[attr(ignore, skip)]
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_err(), "should fail, but ok");

            let err = res.unwrap_err().to_string();
            assert_eq!(err, "duplicated attribute's argument found");
        }
    }

    mod dedup_first {
        use super::*;

        #[derive(Debug, Default, ParseAttrs)]
        struct Attr {
            #[parse(ident, dedup = first, alias = skip)]
            ignore: Option<syn::Ident>,
        }

        #[test]
        fn picks_first_arg() {
            let input: syn::DeriveInput = syn::parse_quote! {
                #[attr(ignore, skip)]
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_ok(), "failed: {}", res.unwrap_err());

            assert_eq!(
                res.unwrap().ignore,
                Some(syn::Ident::new_on_call_site("ignore")),
            );
        }
    }

    mod dedup_last {
        use super::*;

        #[derive(Debug, Default, ParseAttrs)]
        struct Attr {
            #[parse(ident, dedup = last, alias = skip)]
            ignore: Option<syn::Ident>,
        }

        #[test]
        fn picks_last_arg() {
            let input: syn::DeriveInput = syn::parse_quote! {
                #[attr(ignore, skip)]
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_ok(), "failed: {}", res.unwrap_err());

            assert_eq!(
                res.unwrap().ignore,
                Some(syn::Ident::new_on_call_site("skip")),
            );
        }
    }

    mod custom_validation {
        use synthez::proc_macro2::Span;

        use super::*;

        #[derive(Debug, Default, ParseAttrs)]
        struct Attr {
            #[parse(ident, validate = wrong)]
            ignore: Option<syn::Ident>,
        }

        fn wrong(_: &Option<syn::Ident>) -> syn::Result<()> {
            Err(syn::Error::new(Span::call_site(), "wrong!"))
        }

        #[test]
        fn is_invoked() {
            let input: syn::DeriveInput = syn::parse_quote! {
                #[attr(ignore)]
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_err(), "should fail, but ok");

            let err = res.unwrap_err().to_string();
            assert_eq!(err, "wrong!");
        }
    }

    mod raw {
        use synthez::proc_macro2::Span;

        use super::*;

        #[derive(Debug, Default, ParseAttrs)]
        struct Attr {
            #[parse(ident)]
            r#type: Option<syn::token::Type>,
        }

        #[test]
        fn is_unrawed() {
            let input: syn::DeriveInput = syn::parse_quote! {
                #[attr(type)]
                struct Dummy;
            };

            let _ident = syn::Ident::new_on_call_site("type");
            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_ok(), "failed: {}", res.unwrap_err());

            assert_eq!(
                res.unwrap().r#type,
                Some(syn::Token![type](Span::call_site())),
            );
        }
    }
}

mod value {
    use synthez::{IdentExt as _, ParseAttrs, syn};

    mod option {
        use super::*;

        #[derive(Debug, Default, ParseAttrs)]
        struct Attr {
            #[parse(value)]
            name: Option<syn::Ident>,
        }

        #[test]
        fn allows_present() {
            let input: syn::DeriveInput = syn::parse_quote! {
                #[attr(name = minas)]
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_ok(), "failed: {}", res.unwrap_err());

            assert_eq!(
                res.unwrap().name,
                Some(syn::Ident::new_on_call_site("minas")),
            );
        }

        #[test]
        fn allows_absent() {
            let input: syn::DeriveInput = syn::parse_quote! {
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_ok(), "failed: {}", res.unwrap_err());

            assert_eq!(res.unwrap().name, None);
        }
    }

    mod option_fallback {
        use synthez::field;

        use super::*;

        #[derive(Debug, Default, ParseAttrs)]
        struct Attr {
            #[parse(value, fallback = field::if_empty(use_back))]
            name: Option<syn::Ident>,
        }

        fn use_back(_: &[syn::Attribute]) -> syn::Result<Option<syn::Ident>> {
            Ok(Some(syn::Ident::new_on_call_site("fallen")))
        }

        #[test]
        fn omits_not_empty() {
            let input: syn::DeriveInput = syn::parse_quote! {
                #[attr(name = minas)]
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_ok(), "failed: {}", res.unwrap_err());

            assert_eq!(
                res.unwrap().name,
                Some(syn::Ident::new_on_call_site("minas")),
            );
        }

        #[test]
        fn uses_if_empty() {
            let input: syn::DeriveInput = syn::parse_quote! {
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_ok(), "failed: {}", res.unwrap_err());

            assert_eq!(
                res.unwrap().name,
                Some(syn::Ident::new_on_call_site("fallen")),
            );
        }
    }

    mod required {
        use synthez::Required;

        use super::*;

        #[derive(Debug, Default, ParseAttrs)]
        struct Attr {
            #[parse(value)]
            name: Required<syn::Ident>,
        }

        #[test]
        fn allows_present() {
            let input: syn::DeriveInput = syn::parse_quote! {
                #[attr(name = minas)]
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_ok(), "failed: {}", res.unwrap_err());

            assert_eq!(
                *res.unwrap().name,
                syn::Ident::new_on_call_site("minas"),
            );
        }

        #[test]
        fn forbids_absent() {
            let input: syn::DeriveInput = syn::parse_quote! {
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_err(), "should fail, but is ok");

            let err = res.unwrap_err().to_string();
            assert!(
                err.contains(
                    "`name` argument of `#[attr]` attribute is expected",
                ),
                "wrong err:\n{err}",
            );
        }
    }

    mod required_aliased {
        use synthez::Required;

        use super::*;

        #[derive(Debug, Default, ParseAttrs)]
        struct Attr {
            #[parse(value, alias = required)]
            name: Required<syn::Ident>,
        }

        #[test]
        fn allows_present() {
            let input: syn::DeriveInput = syn::parse_quote! {
                #[attr(name = minas)]
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_ok(), "failed: {}", res.unwrap_err());

            assert_eq!(
                *res.unwrap().name,
                syn::Ident::new_on_call_site("minas"),
            );
        }

        #[test]
        fn allows_alias() {
            let input: syn::DeriveInput = syn::parse_quote! {
                #[attr(required = tirith)]
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_ok(), "failed: {}", res.unwrap_err());

            assert_eq!(
                *res.unwrap().name,
                syn::Ident::new_on_call_site("tirith"),
            );
        }

        #[test]
        fn forbids_absent() {
            let input: syn::DeriveInput = syn::parse_quote! {
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_err(), "should fail, but is ok");

            let err = res.unwrap_err().to_string();
            assert!(
                err.contains(
                    "either `name` or `required` argument of `#[attr]` \
                     attribute is expected to be present, but is absent",
                ),
                "wrong err:\n{err}",
            );
        }
    }

    mod required_fallback {
        use synthez::{Required, field};

        use super::*;

        #[derive(Debug, Default, ParseAttrs)]
        struct Attr {
            #[parse(value, fallback = field::if_empty(use_back))]
            name: Required<syn::Ident>,
        }

        fn use_back(_: &[syn::Attribute]) -> syn::Result<Option<syn::Ident>> {
            Ok(Some(syn::Ident::new_on_call_site("fallen")))
        }

        #[test]
        fn omits_not_empty() {
            let input: syn::DeriveInput = syn::parse_quote! {
                #[attr(name = minas)]
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_ok(), "failed: {}", res.unwrap_err());

            assert_eq!(
                *res.unwrap().name,
                syn::Ident::new_on_call_site("minas"),
            );
        }

        #[test]
        fn uses_if_empty() {
            let input: syn::DeriveInput = syn::parse_quote! {
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_ok(), "failed: {}", res.unwrap_err());

            assert_eq!(
                *res.unwrap().name,
                syn::Ident::new_on_call_site("fallen"),
            );
        }
    }

    mod vec {
        use super::*;

        #[derive(Debug, Default, ParseAttrs)]
        struct Attr {
            #[parse(value)]
            names: Vec<syn::Ident>,
        }

        #[test]
        fn allows_present() {
            let input: syn::DeriveInput = syn::parse_quote! {
                #[attr(names(minas, tirith))]
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_ok(), "failed: {}", res.unwrap_err());

            assert_eq!(
                res.unwrap().names,
                vec![
                    syn::Ident::new_on_call_site("minas"),
                    syn::Ident::new_on_call_site("tirith"),
                ],
            );
        }

        #[test]
        fn empty_when_absent() {
            let input: syn::DeriveInput = syn::parse_quote! {
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_ok(), "failed: {}", res.unwrap_err());

            assert_eq!(res.unwrap().names, <Vec<syn::Ident>>::new());
        }
    }

    mod vec_fallback {
        use synthez::field;

        use super::*;

        #[derive(Debug, Default, ParseAttrs)]
        struct Attr {
            #[parse(value, fallback = field::if_empty(use_back))]
            names: Vec<syn::Ident>,
        }

        fn use_back(_: &[syn::Attribute]) -> syn::Result<Option<syn::Ident>> {
            Ok(Some(syn::Ident::new_on_call_site("fallen")))
        }

        #[test]
        fn omits_not_empty() {
            let input: syn::DeriveInput = syn::parse_quote! {
                #[attr(names = minas)]
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_ok(), "failed: {}", res.unwrap_err());

            assert_eq!(
                res.unwrap().names,
                vec![syn::Ident::new_on_call_site("minas")],
            );
        }

        #[test]
        fn uses_if_empty() {
            let input: syn::DeriveInput = syn::parse_quote! {
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_ok(), "failed: {}", res.unwrap_err());

            assert_eq!(
                res.unwrap().names,
                vec![syn::Ident::new_on_call_site("fallen")],
            );
        }
    }

    mod renamed {
        use super::*;

        #[derive(Debug, Default, ParseAttrs)]
        struct Attr {
            #[parse(value, arg = n)]
            name: Option<syn::Ident>,
        }

        #[test]
        fn accepts_new_arg_name() {
            let input: syn::DeriveInput = syn::parse_quote! {
                #[attr(n = minas)]
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_ok(), "failed: {}", res.unwrap_err());

            assert_eq!(
                res.unwrap().name,
                Some(syn::Ident::new_on_call_site("minas")),
            );
        }

        #[test]
        fn errs_on_old_arg_name() {
            let input: syn::DeriveInput = syn::parse_quote! {
                #[attr(name = minas)]
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_err(), "should fail, but ok");

            let err = res.unwrap_err().to_string();
            assert_eq!(err, "unknown `name` attribute argument");
        }
    }

    mod aliased {
        use super::*;

        #[derive(Debug, Default, ParseAttrs)]
        struct Attr {
            #[parse(value, alias = n)]
            name: Option<syn::Ident>,
        }

        #[test]
        fn accepts_alias_arg_name() {
            let input: syn::DeriveInput = syn::parse_quote! {
                #[attr(n = minas)]
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_ok(), "failed: {}", res.unwrap_err());

            assert_eq!(
                res.unwrap().name,
                Some(syn::Ident::new_on_call_site("minas")),
            );
        }

        #[test]
        fn accepts_old_arg_name() {
            let input: syn::DeriveInput = syn::parse_quote! {
                #[attr(name = minas)]
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_ok(), "failed: {}", res.unwrap_err());

            assert_eq!(
                res.unwrap().name,
                Some(syn::Ident::new_on_call_site("minas")),
            );
        }
    }

    mod renamed_and_aliased {
        use super::*;

        #[derive(Debug, Default, ParseAttrs)]
        struct Attr {
            #[parse(value, arg = n, alias = named)]
            name: Option<syn::Ident>,
        }

        #[test]
        fn accepts_new_arg_name() {
            let input: syn::DeriveInput = syn::parse_quote! {
                #[attr(n = minas)]
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_ok(), "failed: {}", res.unwrap_err());

            assert_eq!(
                res.unwrap().name,
                Some(syn::Ident::new_on_call_site("minas")),
            );
        }

        #[test]
        fn accepts_alias_arg_name() {
            let input: syn::DeriveInput = syn::parse_quote! {
                #[attr(named = minas)]
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_ok(), "failed: {}", res.unwrap_err());

            assert_eq!(
                res.unwrap().name,
                Some(syn::Ident::new_on_call_site("minas")),
            );
        }

        #[test]
        fn errs_on_old_arg_name() {
            let input: syn::DeriveInput = syn::parse_quote! {
                #[attr(name = minas)]
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_err(), "should fail, but ok");

            let err = res.unwrap_err().to_string();
            assert_eq!(err, "unknown `name` attribute argument");
        }
    }

    mod dedup_default {
        use super::*;

        #[derive(Debug, Default, ParseAttrs)]
        struct Attr {
            #[parse(value, alias = named)]
            name: Option<syn::Ident>,
        }

        #[test]
        fn is_unique() {
            let input: syn::DeriveInput = syn::parse_quote! {
                #[attr(name = minas, named = tirith)]
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_err(), "should fail, but ok");

            let err = res.unwrap_err().to_string();
            assert_eq!(err, "duplicated attribute's argument found");
        }
    }

    mod dedup_unique {
        use super::*;

        #[derive(Debug, Default, ParseAttrs)]
        struct Attr {
            #[parse(value, dedup = unique, alias = named)]
            names: Vec<syn::Ident>,
        }

        #[test]
        fn forbids_repeated_arg() {
            let input: syn::DeriveInput = syn::parse_quote! {
                #[attr(names(minas, tirith), named = tirith)]
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_err(), "should fail, but ok");

            let err = res.unwrap_err().to_string();
            assert_eq!(err, "duplicated attribute's argument found");
        }
    }

    mod dedup_first {
        use super::*;

        #[derive(Debug, Default, ParseAttrs)]
        struct Attr {
            #[parse(value, dedup = first)]
            name: Option<syn::Ident>,
        }

        #[test]
        fn picks_first_arg() {
            let input: syn::DeriveInput = syn::parse_quote! {
                #[attr(name = minas)]
                #[attr(name = tirith)]
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_ok(), "failed: {}", res.unwrap_err());

            assert_eq!(
                res.unwrap().name,
                Some(syn::Ident::new_on_call_site("minas")),
            );
        }
    }

    mod dedup_last {
        use super::*;

        #[derive(Debug, Default, ParseAttrs)]
        struct Attr {
            #[parse(value, dedup = last)]
            name: Option<syn::Ident>,
        }

        #[test]
        fn picks_last_arg() {
            let input: syn::DeriveInput = syn::parse_quote! {
                #[attr(name(minas, tirith))]
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_ok(), "failed: {}", res.unwrap_err());

            assert_eq!(
                res.unwrap().name,
                Some(syn::Ident::new_on_call_site("tirith")),
            );
        }
    }

    mod custom_validation {
        use synthez::proc_macro2::Span;

        use super::*;

        #[derive(Debug, Default, ParseAttrs)]
        struct Attr {
            #[parse(value, validate = wrong)]
            name: Option<syn::Ident>,
        }

        fn wrong(_: &Option<syn::Ident>) -> syn::Result<()> {
            Err(syn::Error::new(Span::call_site(), "wrong!"))
        }

        #[test]
        fn is_invoked() {
            let input: syn::DeriveInput = syn::parse_quote! {
                #[attr(name = minas)]
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_err(), "should fail, but ok");

            let err = res.unwrap_err().to_string();
            assert_eq!(err, "wrong!");
        }
    }

    mod format {
        use super::*;

        #[derive(Debug, Default, ParseAttrs)]
        struct Attr {
            #[parse(value)]
            name: Option<syn::Ident>,
        }

        #[test]
        fn forbids_no_value() {
            let input: syn::DeriveInput = syn::parse_quote! {
                #[attr(name)]
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_err(), "should fail, but ok");

            let err = res.unwrap_err().to_string();
            assert_eq!(err, "expected `=`");
        }

        #[test]
        fn forbids_not_eq_ident_before_value() {
            let input: syn::DeriveInput = syn::parse_quote! {
                #[attr(name [here])]
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_err(), "should fail, but ok");

            let err = res.unwrap_err().to_string();
            assert_eq!(err, "expected `=`");
        }

        #[test]
        fn allows_parenthesized() {
            let input: syn::DeriveInput = syn::parse_quote! {
                #[attr(name(minas))]
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_ok(), "failed: {}", res.unwrap_err());

            assert_eq!(
                res.unwrap().name,
                Some(syn::Ident::new_on_call_site("minas")),
            );
        }
    }

    mod raw {
        use super::*;

        #[derive(Debug, Default, ParseAttrs)]
        struct Attr {
            #[parse(value)]
            r#type: Option<syn::Ident>,
        }

        #[test]
        fn is_unrawed() {
            let input: syn::DeriveInput = syn::parse_quote! {
                #[attr(type = minas)]
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_ok(), "failed: {}", res.unwrap_err());

            assert_eq!(
                res.unwrap().r#type,
                Some(syn::Ident::new_on_call_site("minas")),
            );
        }
    }
}

mod map {
    use std::collections::HashMap;

    use synthez::{IdentExt as _, ParseAttrs, syn};

    mod hashmap {
        use super::*;

        #[derive(Debug, Default, ParseAttrs)]
        struct Attr {
            #[parse(map)]
            on: HashMap<syn::Ident, syn::LitStr>,
        }

        #[test]
        fn allows_present() {
            let input: syn::DeriveInput = syn::parse_quote! {
                #[attr(on minas = "tirith")]
                #[attr(on loth = "lorien")]
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_ok(), "failed: {}", res.unwrap_err());

            let out = res.unwrap().on;
            assert_eq!(out.len(), 2, "wrong length of {out:?}");
            assert_eq!(
                out.get(&syn::Ident::new_on_call_site("minas"))
                    .map(syn::LitStr::value)
                    .as_deref(),
                Some("tirith"),
                "wrong item of {out:?}",
            );
            assert_eq!(
                out.get(&syn::Ident::new_on_call_site("loth"))
                    .map(syn::LitStr::value)
                    .as_deref(),
                Some("lorien"),
                "wrong item of {out:?}",
            );
        }

        #[test]
        fn empty_on_absent() {
            let input: syn::DeriveInput = syn::parse_quote! {
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_ok(), "failed: {}", res.unwrap_err());

            assert_eq!(
                res.unwrap().on,
                <HashMap<syn::Ident, syn::LitStr>>::new(),
            );
        }
    }

    mod hashmap_fallback {
        use synthez::field;

        use super::*;

        #[derive(Debug, Default, ParseAttrs)]
        struct Attr {
            #[parse(map, fallback = field::if_empty(use_back))]
            on: HashMap<syn::Ident, syn::LitStr>,
        }

        fn use_back(
            _: &[syn::Attribute],
        ) -> syn::Result<Option<(syn::Ident, syn::LitStr)>> {
            Ok(Some((
                syn::Ident::new_on_call_site("fallen"),
                syn::parse_quote!("fall"),
            )))
        }

        #[test]
        fn omits_not_empty() {
            let input: syn::DeriveInput = syn::parse_quote! {
                #[attr(on minas = "tirith")]
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_ok(), "failed: {}", res.unwrap_err());

            let out = res.unwrap().on;
            assert_eq!(out.len(), 1, "wrong length of {out:?}");
            assert_eq!(
                out.get(&syn::Ident::new_on_call_site("minas"))
                    .map(syn::LitStr::value)
                    .as_deref(),
                Some("tirith"),
                "wrong item of {out:?}",
            );
        }

        #[test]
        fn uses_if_empty() {
            let input: syn::DeriveInput = syn::parse_quote! {
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_ok(), "failed: {}", res.unwrap_err());

            let out = res.unwrap().on;
            assert_eq!(out.len(), 1, "wrong length of {out:?}");
            assert_eq!(
                out.get(&syn::Ident::new_on_call_site("fallen"))
                    .map(syn::LitStr::value)
                    .as_deref(),
                Some("fall"),
                "wrong item of {out:?}",
            );
        }
    }

    mod renamed {
        use super::*;

        #[derive(Debug, Default, ParseAttrs)]
        struct Attr {
            #[parse(map, arg = n)]
            on: HashMap<syn::Ident, syn::LitStr>,
        }

        #[test]
        fn accepts_new_arg_name() {
            let input: syn::DeriveInput = syn::parse_quote! {
                #[attr(n minas = "tirith")]
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_ok(), "failed: {}", res.unwrap_err());

            let out = res.unwrap().on;
            assert_eq!(out.len(), 1, "wrong length of {out:?}");
            assert_eq!(
                out.get(&syn::Ident::new_on_call_site("minas"))
                    .map(syn::LitStr::value)
                    .as_deref(),
                Some("tirith"),
                "wrong item of {out:?}",
            );
        }

        #[test]
        fn errs_on_old_arg_name() {
            let input: syn::DeriveInput = syn::parse_quote! {
                #[attr(on minas = "tirith")]
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_err(), "should fail, but ok");

            let err = res.unwrap_err().to_string();
            assert_eq!(err, "unknown `on` attribute argument");
        }
    }

    mod aliased {
        use super::*;

        #[derive(Debug, Default, ParseAttrs)]
        struct Attr {
            #[parse(map, alias = n)]
            on: HashMap<syn::Ident, syn::LitStr>,
        }

        #[test]
        fn accepts_alias_arg_name() {
            let input: syn::DeriveInput = syn::parse_quote! {
                #[attr(n minas = "tirith")]
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_ok(), "failed: {}", res.unwrap_err());

            let out = res.unwrap().on;
            assert_eq!(out.len(), 1, "wrong length of {out:?}");
            assert_eq!(
                out.get(&syn::Ident::new_on_call_site("minas"))
                    .map(syn::LitStr::value)
                    .as_deref(),
                Some("tirith"),
                "wrong item of {out:?}",
            );
        }

        #[test]
        fn accepts_old_arg_name() {
            let input: syn::DeriveInput = syn::parse_quote! {
                #[attr(on minas = "tirith")]
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_ok(), "failed: {}", res.unwrap_err());

            let out = res.unwrap().on;
            assert_eq!(out.len(), 1, "wrong length of {out:?}");
            assert_eq!(
                out.get(&syn::Ident::new_on_call_site("minas"))
                    .map(syn::LitStr::value)
                    .as_deref(),
                Some("tirith"),
                "wrong item of {out:?}",
            );
        }
    }

    mod renamed_and_aliased {
        use super::*;

        #[derive(Debug, Default, ParseAttrs)]
        struct Attr {
            #[parse(map, arg = n, alias = named)]
            on: HashMap<syn::Ident, syn::LitStr>,
        }

        #[test]
        fn accepts_new_arg_name() {
            let input: syn::DeriveInput = syn::parse_quote! {
                #[attr(n minas = "tirith")]
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_ok(), "failed: {}", res.unwrap_err());

            let out = res.unwrap().on;
            assert_eq!(out.len(), 1, "wrong length of {out:?}");
            assert_eq!(
                out.get(&syn::Ident::new_on_call_site("minas"))
                    .map(syn::LitStr::value)
                    .as_deref(),
                Some("tirith"),
                "wrong item of {out:?}",
            );
        }

        #[test]
        fn accepts_alias_arg_name() {
            let input: syn::DeriveInput = syn::parse_quote! {
                #[attr(named minas = "tirith")]
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_ok(), "failed: {}", res.unwrap_err());

            let out = res.unwrap().on;
            assert_eq!(out.len(), 1, "wrong length of {out:?}");
            assert_eq!(
                out.get(&syn::Ident::new_on_call_site("minas"))
                    .map(syn::LitStr::value)
                    .as_deref(),
                Some("tirith"),
                "wrong item of {out:?}",
            );
        }

        #[test]
        fn errs_on_old_arg_name() {
            let input: syn::DeriveInput = syn::parse_quote! {
                #[attr(on minas = "tirith")]
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_err(), "should fail, but ok");

            let err = res.unwrap_err().to_string();
            assert_eq!(err, "unknown `on` attribute argument");
        }
    }

    mod dedup_default {
        use super::*;

        #[derive(Debug, Default, ParseAttrs)]
        struct Attr {
            #[parse(map)]
            on: HashMap<syn::Ident, syn::LitStr>,
        }

        #[test]
        fn is_unique() {
            let input: syn::DeriveInput = syn::parse_quote! {
                #[attr(on minas = "tirith", on minas = "morgul")]
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_err(), "should fail, but ok");

            let err = res.unwrap_err().to_string();
            assert_eq!(err, "duplicated attribute's argument found");
        }
    }

    mod dedup_unique {
        use super::*;

        #[derive(Debug, Default, ParseAttrs)]
        struct Attr {
            #[parse(map, dedup = unique, alias = named)]
            on: HashMap<syn::Ident, syn::LitStr>,
        }

        #[test]
        fn forbids_repeated_arg() {
            let input: syn::DeriveInput = syn::parse_quote! {
                #[attr(on minas = "tirith", named minas = "morgul")]
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_err(), "should fail, but ok");

            let err = res.unwrap_err().to_string();
            assert_eq!(err, "duplicated attribute's argument found");
        }
    }

    mod dedup_first {
        use super::*;

        #[derive(Debug, Default, ParseAttrs)]
        struct Attr {
            #[parse(map, dedup = first, alias = named)]
            on: HashMap<syn::Ident, syn::LitStr>,
        }

        #[test]
        fn picks_first_arg() {
            let input: syn::DeriveInput = syn::parse_quote! {
                #[attr(on minas = "tirith")]
                #[attr(named minas = "morgul")]
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_ok(), "failed: {}", res.unwrap_err());

            let out = res.unwrap().on;
            assert_eq!(out.len(), 1, "wrong length of {out:?}");
            assert_eq!(
                out.get(&syn::Ident::new_on_call_site("minas"))
                    .map(syn::LitStr::value)
                    .as_deref(),
                Some("tirith"),
                "wrong item of {out:?}",
            );
        }
    }

    mod dedup_last {
        use super::*;

        #[derive(Debug, Default, ParseAttrs)]
        struct Attr {
            #[parse(map, dedup = last, alias = named)]
            on: HashMap<syn::Ident, syn::LitStr>,
        }

        #[test]
        fn picks_last_arg() {
            let input: syn::DeriveInput = syn::parse_quote! {
                #[attr(on minas = "tirith", named minas = "morgul")]
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_ok(), "failed: {}", res.unwrap_err());

            let out = res.unwrap().on;
            assert_eq!(out.len(), 1, "wrong length of {out:?}");
            assert_eq!(
                out.get(&syn::Ident::new_on_call_site("minas"))
                    .map(syn::LitStr::value)
                    .as_deref(),
                Some("morgul"),
                "wrong item of {out:?}",
            );
        }
    }

    mod custom_validation {
        use synthez::proc_macro2::Span;

        use super::*;

        #[derive(Debug, Default, ParseAttrs)]
        struct Attr {
            #[parse(map, validate = wrong)]
            on: HashMap<syn::Ident, syn::LitStr>,
        }

        fn wrong(_: &HashMap<syn::Ident, syn::LitStr>) -> syn::Result<()> {
            Err(syn::Error::new(Span::call_site(), "wrong!"))
        }

        #[test]
        fn is_invoked() {
            let input: syn::DeriveInput = syn::parse_quote! {
                #[attr(on minas = "tirith")]
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_err(), "should fail, but ok");

            let err = res.unwrap_err().to_string();
            assert_eq!(err, "wrong!");
        }
    }

    mod format {
        use super::*;

        #[derive(Debug, Default, ParseAttrs)]
        struct Attr {
            #[parse(map)]
            on: HashMap<syn::Ident, syn::LitStr>,
        }

        #[test]
        fn forbids_no_key_value() {
            let input: syn::DeriveInput = syn::parse_quote! {
                #[attr(on)]
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_err(), "should fail, but ok");

            let err = res.unwrap_err().to_string();
            assert_eq!(err, "unexpected end of input, expected identifier");
        }

        #[test]
        fn forbids_no_key() {
            let input: syn::DeriveInput = syn::parse_quote! {
                #[attr(on = "tirith")]
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_err(), "should fail, but ok");

            let err = res.unwrap_err().to_string();
            assert_eq!(err, "expected identifier");
        }

        #[test]
        fn forbids_no_value() {
            let input: syn::DeriveInput = syn::parse_quote! {
                #[attr(on minas)]
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_err(), "should fail, but ok");

            let err = res.unwrap_err().to_string();
            assert_eq!(err, "expected `=`");
        }

        #[test]
        fn forbids_not_eq_ident_before_value() {
            let input: syn::DeriveInput = syn::parse_quote! {
                #[attr(on minas "tirith")]
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_err(), "should fail, but ok");

            let err = res.unwrap_err().to_string();
            assert_eq!(err, "expected `=`");
        }

        #[test]
        fn forbids_parenthesized_key() {
            let input: syn::DeriveInput = syn::parse_quote! {
                #[attr(on (minas = "tirith"))]
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_err(), "should fail, but ok");

            let err = res.unwrap_err().to_string();
            assert_eq!(err, "expected identifier");
        }

        #[test]
        fn forbids_parenthesized_value() {
            let input: syn::DeriveInput = syn::parse_quote! {
                #[attr(on minas("tirith"))]
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_err(), "should fail, but ok");

            let err = res.unwrap_err().to_string();
            assert_eq!(err, "expected `=`");
        }
    }

    mod raw {
        use super::*;

        #[derive(Debug, Default, ParseAttrs)]
        struct Attr {
            #[parse(map)]
            r#type: HashMap<syn::Ident, syn::LitStr>,
        }

        #[test]
        fn is_unrawed() {
            let input: syn::DeriveInput = syn::parse_quote! {
                #[attr(type minas = "tirith")]
                #[attr(type loth = "lorien")]
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_ok(), "failed: {}", res.unwrap_err());

            let out = res.unwrap().r#type;
            assert_eq!(out.len(), 2, "wrong length of {out:?}");
            assert_eq!(
                out.get(&syn::Ident::new_on_call_site("minas"))
                    .map(syn::LitStr::value)
                    .as_deref(),
                Some("tirith"),
                "wrong item of {out:?}",
            );
            assert_eq!(
                out.get(&syn::Ident::new_on_call_site("loth"))
                    .map(syn::LitStr::value)
                    .as_deref(),
                Some("lorien"),
                "wrong item of {out:?}",
            );
        }
    }
}

mod nested {
    use synthez::{IdentExt as _, ParseAttrs, Spanning, syn};

    mod option {
        use super::*;

        #[derive(Debug, Default, ParseAttrs)]
        struct Sub {
            #[parse(ident)]
            ignore: Option<syn::Ident>,
        }

        #[derive(Debug, Default, ParseAttrs)]
        struct Attr {
            #[parse(nested)]
            sub: Option<Spanning<Sub>>,
        }

        #[test]
        fn allows_present() {
            let input: syn::DeriveInput = syn::parse_quote! {
                #[attr(sub(ignore))]
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_ok(), "failed: {}", res.unwrap_err());

            let out = res.unwrap();
            assert!(out.sub.is_some(), "outer absent, but shouldn't: {out:?}");
            assert_eq!(
                out.sub.unwrap().ignore,
                Some(syn::Ident::new_on_call_site("ignore")),
            );
        }

        #[test]
        fn empty_on_absent_inner() {
            let input: syn::DeriveInput = syn::parse_quote! {
                #[attr(sub())]
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_ok(), "failed: {}", res.unwrap_err());

            let out = res.unwrap();
            assert!(out.sub.is_some(), "outer absent, but shouldn't: {out:?}");
            assert!(
                out.sub.as_ref().unwrap().ignore.is_none(),
                "inner present, but shouldn't: {out:?}",
            );
        }

        #[test]
        fn empty_on_absent_outer() {
            let input: syn::DeriveInput = syn::parse_quote! {
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_ok(), "failed: {}", res.unwrap_err());

            let out = res.unwrap();
            assert!(out.sub.is_none(), "outer present, but shouldn't: {out:?}");
        }
    }

    mod option_fallback {
        use synthez::{field, proc_macro2::Span};

        use super::*;

        #[derive(Debug, Default, ParseAttrs)]
        struct Sub {
            #[parse(ident, fallback = field::if_empty(use_inner_back))]
            ignore: Option<syn::Ident>,
        }

        fn use_inner_back(
            _: &[syn::Attribute],
        ) -> syn::Result<Option<syn::Ident>> {
            Ok(Some(syn::Ident::new_on_call_site("fallen")))
        }

        #[derive(Debug, Default, ParseAttrs)]
        struct Attr {
            #[parse(nested, fallback = field::if_empty(use_outer_back))]
            sub: Option<Spanning<Sub>>,
        }

        fn use_outer_back(
            _: &[syn::Attribute],
        ) -> syn::Result<Option<Spanning<Sub>>> {
            Ok(Some(Spanning::new(
                Sub { ignore: Some(syn::Ident::new_on_call_site("fall")) },
                Span::call_site(),
            )))
        }

        #[test]
        fn omits_not_empty() {
            let input: syn::DeriveInput = syn::parse_quote! {
                #[attr(sub(ignore))]
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_ok(), "failed: {}", res.unwrap_err());

            let out = res.unwrap();
            assert!(out.sub.is_some(), "outer absent, but shouldn't: {out:?}");
            assert_eq!(
                out.sub.unwrap().ignore,
                Some(syn::Ident::new_on_call_site("ignore")),
            );
        }

        #[test]
        fn uses_if_inner_empty() {
            let input: syn::DeriveInput = syn::parse_quote! {
                #[attr(sub())]
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_ok(), "failed: {}", res.unwrap_err());

            let out = res.unwrap();
            assert!(out.sub.is_some(), "outer absent, but shouldn't: {out:?}");
            assert_eq!(
                out.sub.unwrap().ignore,
                Some(syn::Ident::new_on_call_site("fallen")),
            );
        }

        #[test]
        fn uses_if_outer_empty() {
            let input: syn::DeriveInput = syn::parse_quote! {
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_ok(), "failed: {}", res.unwrap_err());

            let out = res.unwrap();
            assert!(out.sub.is_some(), "outer absent, but shouldn't: {out:?}");
            assert_eq!(
                out.sub.unwrap().ignore,
                Some(syn::Ident::new_on_call_site("fall")),
            );
        }
    }

    mod required {
        use synthez::Required;

        use super::*;

        #[derive(Debug, Default, ParseAttrs)]
        struct Sub {
            #[parse(ident)]
            ignore: Required<syn::Ident>,
        }

        #[derive(Debug, Default, ParseAttrs)]
        struct Attr {
            #[parse(nested)]
            sub: Required<Spanning<Sub>>,
        }

        #[test]
        fn allows_present() {
            let input: syn::DeriveInput = syn::parse_quote! {
                #[attr(sub(ignore))]
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_ok(), "failed: {}", res.unwrap_err());

            let out = res.unwrap();
            assert_eq!(*out.sub.ignore, syn::Ident::new_on_call_site("ignore"));
        }

        #[test]
        fn forbids_absent_inner() {
            let input: syn::DeriveInput = syn::parse_quote! {
                #[attr(sub())]
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_err(), "should fail, but is ok");

            let err = res.unwrap_err().to_string();
            assert!(
                err.contains(
                    "`ignore` argument of `#[attr(sub)]` attribute is expected",
                ),
                "wrong err:\n{err}",
            );
        }

        #[test]
        fn forbids_absent_outer() {
            let input: syn::DeriveInput = syn::parse_quote! {
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_err(), "should fail, but is ok");

            let err = res.unwrap_err().to_string();
            assert!(
                err.contains(
                    "`sub` argument of `#[attr]` attribute is expected",
                ),
                "wrong err:\n{err}",
            );
        }
    }

    mod vec {
        use super::*;

        #[derive(Debug, Default, Eq, ParseAttrs, PartialEq)]
        struct Sub {
            #[parse(ident, alias = skip)]
            ignore: Option<syn::Ident>,
        }

        #[derive(Debug, Default, ParseAttrs)]
        struct Attr {
            #[parse(nested)]
            sub: Vec<Spanning<Sub>>,
        }

        #[test]
        fn allows_present() {
            let input: syn::DeriveInput = syn::parse_quote! {
                #[attr(sub(ignore), sub(skip))]
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_ok(), "failed: {}", res.unwrap_err());

            let out = res.unwrap().sub;
            assert_eq!(out.len(), 2, "wrong length of {out:?}");
            assert_eq!(
                out[0].ignore,
                Some(syn::Ident::new_on_call_site("ignore")),
                "wrong item of {out:?}",
            );
            assert_eq!(
                out[1].ignore,
                Some(syn::Ident::new_on_call_site("skip")),
                "wrong item of {out:?}",
            );
        }
    }

    mod renamed {
        use super::*;

        #[derive(Debug, Default, ParseAttrs)]
        struct Sub {
            #[parse(ident, arg = i)]
            ignore: Option<syn::Ident>,
        }

        #[derive(Debug, Default, ParseAttrs)]
        struct Attr {
            #[parse(nested, arg = s)]
            sub: Option<Spanning<Sub>>,
        }

        #[test]
        fn accepts_new_arg_name() {
            let input: syn::DeriveInput = syn::parse_quote! {
                #[attr(s(i))]
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_ok(), "failed: {}", res.unwrap_err());

            let out = res.unwrap();
            assert!(out.sub.is_some(), "outer absent, but shouldn't: {out:?}");
            assert_eq!(
                out.sub.unwrap().ignore,
                Some(syn::Ident::new_on_call_site("i")),
            );
        }

        #[test]
        fn errs_on_old_inner_arg_name() {
            let input: syn::DeriveInput = syn::parse_quote! {
                #[attr(s(ignore))]
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_err(), "should fail, but ok");

            let err = res.unwrap_err().to_string();
            assert_eq!(err, "unknown `ignore` attribute argument");
        }

        #[test]
        fn errs_on_old_outer_arg_name() {
            let input: syn::DeriveInput = syn::parse_quote! {
                #[attr(sub(i))]
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_err(), "should fail, but ok");

            let err = res.unwrap_err().to_string();
            assert_eq!(err, "unknown `sub` attribute argument");
        }
    }

    mod aliased {
        use super::*;

        #[derive(Debug, Default, ParseAttrs)]
        struct Sub {
            #[parse(ident, alias = i)]
            ignore: Option<syn::Ident>,
        }

        #[derive(Debug, Default, ParseAttrs)]
        struct Attr {
            #[parse(nested, alias = s)]
            sub: Option<Spanning<Sub>>,
        }

        #[test]
        fn accepts_alias_arg_name() {
            let input: syn::DeriveInput = syn::parse_quote! {
                #[attr(s(i))]
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_ok(), "failed: {}", res.unwrap_err());

            let out = res.unwrap();
            assert!(out.sub.is_some(), "outer absent, but shouldn't: {out:?}");
            assert_eq!(
                out.sub.unwrap().ignore,
                Some(syn::Ident::new_on_call_site("i")),
            );
        }

        #[test]
        fn accepts_old_inner_arg_name() {
            let input: syn::DeriveInput = syn::parse_quote! {
                #[attr(s(ignore))]
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_ok(), "failed: {}", res.unwrap_err());

            let out = res.unwrap();
            assert!(out.sub.is_some(), "outer absent, but shouldn't: {out:?}");
            assert_eq!(
                out.sub.unwrap().ignore,
                Some(syn::Ident::new_on_call_site("ignore")),
            );
        }

        #[test]
        fn accepts_old_outer_arg_name() {
            let input: syn::DeriveInput = syn::parse_quote! {
                #[attr(sub(i))]
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_ok(), "failed: {}", res.unwrap_err());

            let out = res.unwrap();
            assert!(out.sub.is_some(), "outer absent, but shouldn't: {out:?}");
            assert_eq!(
                out.sub.unwrap().ignore,
                Some(syn::Ident::new_on_call_site("i")),
            );
        }
    }

    mod renamed_and_aliased {
        use super::*;

        #[derive(Debug, Default, ParseAttrs)]
        struct Sub {
            #[parse(ident, arg = i, alias = skip)]
            ignore: Option<syn::Ident>,
        }

        #[derive(Debug, Default, ParseAttrs)]
        struct Attr {
            #[parse(nested, arg = s, alias = child)]
            sub: Option<Spanning<Sub>>,
        }

        #[test]
        fn accepts_new_arg_name() {
            let input: syn::DeriveInput = syn::parse_quote! {
                #[attr(s(i))]
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_ok(), "failed: {}", res.unwrap_err());

            let out = res.unwrap();
            assert!(out.sub.is_some(), "outer absent, but shouldn't: {out:?}");
            assert_eq!(
                out.sub.unwrap().ignore,
                Some(syn::Ident::new_on_call_site("i")),
            );
        }

        #[test]
        fn accepts_alias_arg_name() {
            let input: syn::DeriveInput = syn::parse_quote! {
                #[attr(child(skip))]
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_ok(), "failed: {}", res.unwrap_err());

            let out = res.unwrap();
            assert!(out.sub.is_some(), "outer absent, but shouldn't: {out:?}");
            assert_eq!(
                out.sub.unwrap().ignore,
                Some(syn::Ident::new_on_call_site("skip")),
            );
        }

        #[test]
        fn errs_on_old_inner_arg_name() {
            let input: syn::DeriveInput = syn::parse_quote! {
                #[attr(s(ignore))]
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_err(), "should fail, but ok");

            let err = res.unwrap_err().to_string();
            assert_eq!(err, "unknown `ignore` attribute argument");
        }

        #[test]
        fn errs_on_old_outer_arg_name() {
            let input: syn::DeriveInput = syn::parse_quote! {
                #[attr(sub(skip))]
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_err(), "should fail, but ok");

            let err = res.unwrap_err().to_string();
            assert_eq!(err, "unknown `sub` attribute argument");
        }
    }

    mod dedup_default {
        use super::*;

        #[derive(Debug, Default, ParseAttrs)]
        struct Sub {
            #[parse(ident, alias = skip)]
            ignore: Option<syn::Ident>,
        }

        #[derive(Debug, Default, ParseAttrs)]
        struct Attr {
            #[parse(nested)]
            sub: Option<Spanning<Sub>>,
        }

        #[test]
        fn is_unique() {
            let input: syn::DeriveInput = syn::parse_quote! {
                #[attr(sub(ignore), sub(skip))]
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_err(), "should fail, but ok");

            let err = res.unwrap_err().to_string();
            assert_eq!(err, "duplicated attribute's argument found");
        }
    }

    mod dedup_unique {
        use super::*;

        #[derive(Debug, Default, ParseAttrs)]
        struct Sub {
            #[parse(ident, dedup = unique, alias = skip)]
            ignore: Option<syn::Ident>,
        }

        #[derive(Debug, Default, ParseAttrs)]
        struct Attr {
            #[parse(nested, dedup = unique, alias = child)]
            sub: Option<Spanning<Sub>>,
        }

        #[test]
        fn forbids_repeated_arg() {
            let input: syn::DeriveInput = syn::parse_quote! {
                #[attr(child(ignore))]
                #[attr(sub(skip))]
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_err(), "should fail, but ok");

            let err = res.unwrap_err().to_string();
            assert_eq!(err, "duplicated attribute's argument found");
        }
    }

    mod dedup_first {
        use super::*;

        #[derive(Debug, Default, ParseAttrs)]
        struct Sub {
            #[parse(ident, alias = skip)]
            ignore: Option<syn::Ident>,
        }

        #[derive(Debug, Default, ParseAttrs)]
        struct Attr {
            #[parse(nested, dedup = first, alias = child)]
            sub: Option<Spanning<Sub>>,
        }

        #[test]
        fn picks_first_arg() {
            let input: syn::DeriveInput = syn::parse_quote! {
                #[attr(child(skip))]
                #[attr(sub(ignore))]
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_ok(), "failed: {}", res.unwrap_err());

            let out = res.unwrap();
            assert!(out.sub.is_some(), "outer absent, but shouldn't: {out:?}");
            assert_eq!(
                out.sub.unwrap().ignore,
                Some(syn::Ident::new_on_call_site("skip")),
            );
        }
    }

    mod dedup_last {
        use super::*;

        #[derive(Debug, Default, ParseAttrs)]
        struct Sub {
            #[parse(ident, alias = skip)]
            ignore: Option<syn::Ident>,
        }

        #[derive(Debug, Default, ParseAttrs)]
        struct Attr {
            #[parse(nested, dedup = last, alias = child)]
            sub: Option<Spanning<Sub>>,
        }

        #[test]
        fn picks_last_arg() {
            let input: syn::DeriveInput = syn::parse_quote! {
                #[attr(child(skip))]
                #[attr(sub(ignore))]
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_ok(), "failed: {}", res.unwrap_err());

            let out = res.unwrap();
            assert!(out.sub.is_some(), "outer absent, but shouldn't: {out:?}");
            assert_eq!(
                out.sub.unwrap().ignore,
                Some(syn::Ident::new_on_call_site("ignore")),
            );
        }
    }

    mod custom_validation {
        use synthez::proc_macro2::Span;

        use super::*;

        #[derive(Debug, Default, ParseAttrs)]
        struct Sub {
            #[parse(ident, validate = wrong_inner)]
            ignore: Option<syn::Ident>,
        }

        fn wrong_inner(id: &Option<syn::Ident>) -> syn::Result<()> {
            if id.is_none() {
                return Err(syn::Error::new(Span::call_site(), "wrong inner!"));
            }
            Ok(())
        }

        #[derive(Debug, Default, ParseAttrs)]
        struct Attr {
            #[parse(nested, validate = wrong_outer)]
            sub: Option<Spanning<Sub>>,
        }

        fn wrong_outer(_: &Option<Spanning<Sub>>) -> syn::Result<()> {
            Err(syn::Error::new(Span::call_site(), "wrong outer!"))
        }

        #[test]
        fn is_invoked_on_inner() {
            let input: syn::DeriveInput = syn::parse_quote! {
                #[attr(sub())]
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_err(), "should fail, but ok");

            let err = res.unwrap_err().to_string();
            assert_eq!(err, "wrong inner!");
        }

        #[test]
        fn is_invoked_on_outer() {
            let input: syn::DeriveInput = syn::parse_quote! {
                #[attr(sub(ignore))]
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_err(), "should fail, but ok");

            let err = res.unwrap_err().to_string();
            assert_eq!(err, "wrong outer!");
        }
    }

    mod format {
        use super::*;

        #[derive(Debug, Default, ParseAttrs)]
        struct Sub {
            #[parse(ident)]
            ignore: Option<syn::Ident>,
        }

        #[derive(Debug, Default, ParseAttrs)]
        struct Attr {
            #[parse(nested)]
            sub: Option<Spanning<Sub>>,
        }

        #[test]
        fn forbids_no_parentheses() {
            let input: syn::DeriveInput = syn::parse_quote! {
                #[attr(sub)]
                struct Dummy;
            };

            let res = Attr::parse_attrs("attr", &input);
            assert!(res.is_err(), "should fail, but ok");

            let err = res.unwrap_err().to_string();
            assert_eq!(err, "unexpected end of input, expected parentheses");
        }
    }
}
