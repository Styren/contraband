use proc_macro2::{Ident, Span};
use std::collections::HashMap;
use syn::Meta;

pub(crate) struct ModuleArgs {
    pub(crate) controllers: Vec<syn::Path>,
    pub(crate) imports: Vec<syn::Path>,
    pub(crate) exports: Vec<syn::Path>,
    pub(crate) providers: Vec<syn::Path>,
}

impl ModuleArgs {
    pub(crate) fn parse_and_strip(attrs: &mut std::vec::Vec<syn::Attribute>) -> syn::Result<Self> {
        let mut controllers = Vec::new();
        let mut imports = Vec::new();
        let mut exports = Vec::new();
        let mut providers = Vec::new();
        let mut path_to_vec = HashMap::new();
        let call_site = Span::call_site();
        path_to_vec.insert(Ident::new("controller", call_site), &mut controllers);
        path_to_vec.insert(Ident::new("import", call_site), &mut imports);
        path_to_vec.insert(Ident::new("export", call_site), &mut exports);
        path_to_vec.insert(Ident::new("provider", call_site), &mut providers);
        for attr in attrs.clone() {
            match attr.parse_meta() {
                Ok(Meta::List(nv)) => {
                    if let Some(vec) = nv.path.get_ident().and_then(|x| path_to_vec.get_mut(x)) {
                        for item in nv.nested {
                            if let syn::NestedMeta::Meta(syn::Meta::Path(path)) = item {
                                vec.push(path);
                            } else {
                                return Err(syn::Error::new_spanned(
                                    nv.path,
                                    "Attribute guard expects a path!",
                                ));
                            }
                        }
                    } else {
                        return Err(syn::Error::new_spanned(
                            nv.path,
                            "Unknown attribute key is specified.",
                        ));
                    }
                }
                Ok(Meta::Path(path)) => {
                    return Err(syn::Error::new_spanned(
                        path,
                        "Unknown attribute key is specified.",
                    ))
                }
                Ok(arg) => {
                    return Err(syn::Error::new_spanned(arg, "Unknown attribute."));
                }
                Err(_) => {}
            }
        }
        attrs.retain(|attr| {
            attr.path
                .get_ident()
                .and_then(|x| path_to_vec.get(x))
                .is_none()
        });
        Ok(Self {
            controllers,
            imports,
            exports,
            providers,
        })
    }
}
