use crate::Spanning;

pub fn doc_string(
    attrs: &[syn::Attribute],
) -> syn::Result<Option<Spanning<String>>> {
    let mut span = None;
    let mut out = String::new();

    for a in attrs {
        match a.parse_meta()? {
            syn::Meta::NameValue(item) if item.path.is_ident("doc") => {
                match item.lit {
                    syn::Lit::Str(lit) => {
                        if span.is_none() {
                            span = Some(lit.span());
                        }

                        let s = lit.value();
                        let s = s.strip_prefix(' ').unwrap_or(&s).trim_end();
                        if s.ends_with('\\') {
                            out.push_str(s.trim_end_matches('\\'));
                            out.push(' ');
                        } else {
                            out.push_str(s);
                            out.push('\n');
                        }
                    }
                    _ => return Err(syn::Error::new(
                        item.lit.span(),
                        "`#[doc]` attribute can contain string literals only",
                    )),
                }
            }
            _ => {}
        }
    }
    if !out.is_empty() {
        out.truncate(out.trim_end().len())
    }

    Ok(span.map(|s| Spanning::new(out, s)))
}

#[inline]
pub fn doc(attrs: &[syn::Attribute]) -> syn::Result<Option<syn::LitStr>> {
    doc_string(attrs).map(|opt| opt.map(Into::into))
}
