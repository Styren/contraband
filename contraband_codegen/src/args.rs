use syn::{AttributeArgs, NestedMeta};
use proc_macro2::Span;

pub(crate) struct Args {
    pub(crate) path: syn::LitStr,
    pub(crate) guards: Vec<syn::Path>,
    pub(crate) wrappers: Vec<syn::Path>,
}

impl Args {
    pub(crate) fn new(args: AttributeArgs) -> syn::Result<Self> {
        let mut path = None;
        let mut guards = Vec::new();
        let mut wrappers = Vec::new();
        for arg in args {
            match arg {
                NestedMeta::Lit(syn::Lit::Str(lit)) => match path {
                    None => {
                        path = Some(lit);
                    }
                    _ => {
                        return Err(syn::Error::new_spanned(
                            lit,
                            "Multiple paths specified! Should be only one!",
                        ));
                    }
                },
                NestedMeta::Meta(syn::Meta::List(nv)) => {
                    if nv.path.is_ident("guard") {
                        for guard in nv.nested {
                            if let syn::NestedMeta::Meta(syn::Meta::Path(path)) = guard {
                                guards.push(path);
                            } else {
                                return Err(syn::Error::new_spanned(
                                        nv.path,
                                        "Attribute guard expects a path!",
                                ));
                            }
                        }
                    } else if nv.path.is_ident("wrap") {
                        for wrapper in nv.nested {
                            if let syn::NestedMeta::Meta(syn::Meta::Path(path)) = wrapper {
                                wrappers.push(path);
                            } else {
                                return Err(syn::Error::new_spanned(
                                        nv.path,
                                "Attribute wrap expects type",
                                ));
                            }
                        }
                    }
                },
                NestedMeta::Meta(syn::Meta::NameValue(nv)) => {
                    if nv.path.is_ident("path") {
                        if let syn::Lit::Str(lit) = nv.lit {
                            path = Some(lit);
                        } else {
                            return Err(syn::Error::new_spanned(
                                nv.lit,
                                "Path expects literal string.",
                            ));
                        }
                    } else {
                        return Err(syn::Error::new_spanned(
                            nv.path,
                            "Unknown attribute key is specified.",
                        ));
                    }
                }
                arg => {
                    return Err(syn::Error::new_spanned(arg, "Unknown attribute."));
                }
            }
        }
        Ok(Args {
            path: path.unwrap_or_else(|| syn::LitStr::new("/", proc_macro2::Span::call_site())),
            guards,
            wrappers,
        })
    }
}

impl Default for Args {
    fn default() -> Self {
        Self {
            path: syn::LitStr::new("", Span::call_site()),
            guards: Vec::new(),
            wrappers: Vec::new(),
        }
    }
}
