use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::Ident;

impl<'a> ToTokens for InjectedBody<'a> {
    fn to_tokens(&self, stream: &mut TokenStream2) {
        let Self {
            graph_ident,
            imported_graph_ident,
            fields,
        } = self;
        for field in fields {
            let ident = &field.ident;
            let ty = &field.ty;
            let out = quote! {
                #ident: #graph_ident
                    .get_node::<#ty>()
                    .or_else(|| contraband::graph::Graph::search_all(#imported_graph_ident))
                    .unwrap()
                    .to_owned(),
            };
            stream.extend(out);
        }
    }
}

pub(crate) struct InjectedBody<'a> {
    graph_ident: &'a Ident,
    imported_graph_ident: &'a Ident,
    fields: Vec<syn::Field>,
}

impl<'a> InjectedBody<'a> {
    pub(crate) fn new(
        graph_ident: &'a Ident,
        imported_graph_ident: &'a Ident,
        data: &syn::DataStruct,
    ) -> syn::Result<Self> {
        let mut fields = Vec::new();
        match &data.fields {
            syn::Fields::Named(fl) => {
                for field in fl.named.iter() {
                    fields.push(field.to_owned());
                }
            }
            syn::Fields::Unit => {}
            fields => {
                return Err(syn::Error::new_spanned(
                    fields,
                    "Unnamed structs are not allowed.",
                ));
            }
        }
        Ok(Self {
            graph_ident,
            imported_graph_ident,
            fields,
        })
    }
}
