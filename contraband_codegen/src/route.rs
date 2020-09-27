use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{ToTokens, TokenStreamExt};
use std::str::FromStr;
use syn::Ident;

#[derive(PartialEq)]
pub(crate) enum GuardType {
    Get,
    Post,
    Put,
    Delete,
    Head,
    Connect,
    Options,
    Trace,
    Patch,
}

impl GuardType {
    fn as_guard(&self) -> &'static str {
        match self {
            GuardType::Get => "Get",
            GuardType::Post => "Post",
            GuardType::Put => "Put",
            GuardType::Delete => "Delete",
            GuardType::Head => "Head",
            GuardType::Connect => "Connect",
            GuardType::Options => "Options",
            GuardType::Trace => "Trace",
            GuardType::Patch => "Patch",
        }
    }
}

impl FromStr for GuardType {
    type Err = ();

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "get" => Ok(GuardType::Get),
            "post" => Ok(GuardType::Post),
            "put" => Ok(GuardType::Put),
            "delete" => Ok(GuardType::Delete),
            "head" => Ok(GuardType::Head),
            "connect" => Ok(GuardType::Connect),
            "options" => Ok(GuardType::Options),
            "trace" => Ok(GuardType::Trace),
            "patch" => Ok(GuardType::Patch),
            _ => Err(()),
        }
    }
}

impl ToTokens for GuardType {
    fn to_tokens(&self, stream: &mut TokenStream2) {
        let ident = Ident::new(self.as_guard(), Span::call_site());
        stream.append(ident);
    }
}
