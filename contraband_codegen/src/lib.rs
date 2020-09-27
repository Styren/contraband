//! Contraband codegen module.
//!
//! Generators for controllers, modules and route handlers.
//!
//! ## Documentation & community resources
//!
//! * [GitHub repository](https://github.com/styren/contraband)
//! * [Examples](https://github.com/styren/contraband/tree/master/examples)
extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::Span;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote, ToTokens};
use syn::{parse_macro_input, DeriveInput, Ident, ItemImpl, ItemStruct};
mod args;
mod injected;
mod module;
mod route;
use crate::injected::InjectedBody;
use crate::module::ModuleArgs;
use crate::route::GuardType;
use args::Args;
use std::str::FromStr;

/// Marks function to be run in an async runtime
#[proc_macro_attribute]
pub fn main(_: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = syn::parse_macro_input!(item as syn::ItemFn);
    let attrs = &input.attrs;
    let vis = &input.vis;
    let sig = &mut input.sig;
    let body = &input.block;
    let name = &sig.ident;

    if sig.asyncness.is_none() {
        return syn::Error::new_spanned(sig.fn_token, "only async fn is supported")
            .to_compile_error()
            .into();
    }

    sig.asyncness = None;

    (quote! {
        #(#attrs)*
        #vis #sig {
            contraband::Runtime::new(stringify!(#name))
                .block_on(async move { #body })
        }
    })
    .into()
}

/// Marks test function to be run in an async runtime
#[proc_macro_attribute]
pub fn test(_: TokenStream, item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::ItemFn);

    let ret = &input.sig.output;
    let name = &input.sig.ident;
    let body = &input.block;
    let attrs = &input.attrs;
    let mut has_test_attr = false;

    for attr in attrs {
        if attr.path.is_ident("test") {
            has_test_attr = true;
        }
    }

    if input.sig.asyncness.is_none() {
        return syn::Error::new_spanned(
            input.sig.fn_token,
            format!("only async fn is supported, {}", input.sig.ident),
        )
        .to_compile_error()
        .into();
    }

    let result = if has_test_attr {
        quote! {
            #(#attrs)*
            fn #name() #ret {
                contraband::Runtime::new("test")
                    .block_on(async { #body })
            }
        }
    } else {
        quote! {
            #[test]
            #(#attrs)*
            fn #name() #ret {
                contraband::Runtime::new("test")
                    .block_on(async { #body })
            }
        }
    };

    result.into()
}

/// Derives the `Injectable` trait for dependency injection.
#[proc_macro_derive(Injectable)]
pub fn injectable(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let name = &ast.ident;
    let graph_ident = Ident::new("graph", Span::call_site());
    let context_ident = Ident::new("ctx", Span::call_site());
    let fields = match &ast.data {
        syn::Data::Struct(st) => match InjectedBody::new(&graph_ident, &context_ident, st) {
            Ok(fields) => Ok(fields),
            err => err,
        },
        _ => Err(syn::Error::new_spanned(
            &ast,
            "Can only be applied to structs",
        )),
    };
    match fields {
        Ok(fi) => {
            let expanded = quote! {
                #[automatically_derived]
                impl contraband::graph::Injected for #name {
                    type Output = Self;
                    fn resolve(
                        #graph_ident: &mut contraband::graph::Graph,
                        #context_ident: &[&contraband::graph::Graph]
                    ) -> Self {
                        Self {
                            #fi
                        }
                    }
                }
            };
            TokenStream::from(expanded)
        }
        Err(err) => err.to_compile_error().into(),
    }
}

/// Creates a module.
///
/// Syntax: `#[module]`
///
/// ## Example
///
/// ```rust,no_run
/// use contraband::core::ContrabandApp;
/// use contraband::module;
/// use contraband::{Injectable, controller};
/// use contraband::core::ContrabandApp;
/// use actix_web::HttpResponse;
///
/// #[derive(Clone, Injectable)]
/// struct HelloController;
///
/// #[controller]
/// impl HelloController {
///     #[get]
///     async fn hello_world(self) -> HttpResponse {
///         HttpResponse::Ok().body("Hello world!")
///     }
/// }
///
/// #[module]
/// #[controller(HelloController)]
/// struct AppModule;
///
/// #[contraband::main]
/// async fn main() -> std::io::Result<()> {
///     ContrabandApp::new()
///         .start::<AppModule>()
///         .await
/// }
/// ```
#[proc_macro_attribute]
pub fn module(_: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as ItemStruct);
    let name = &input.ident;
    match ModuleArgs::parse_and_strip(&mut input.attrs) {
        Ok(ModuleArgs {
            controllers,
            imports,
            exports,
            providers,
        }) => {
            let expanded = quote! {
                #input

                #[automatically_derived]
                impl contraband::module::ModuleFactory for #name {
                    fn get_module() -> contraband::module::Module {
                        contraband::module::Module::new()
                            #(.import::<#imports>())*
                            #(.export::<#exports>())*
                            #(.provide::<#providers>())*
                            #(.controller::<#controllers>())*
                    }
                }
            };
            TokenStream::from(expanded)
        }
        Err(err) => err.to_compile_error().into(),
    }
}

struct Method {
    name: Ident,
    guard_type: GuardType,
    args: Args,
    impl_item: syn::ImplItemMethod,
}

impl Method {
    fn new(impl_item: &mut syn::ImplItemMethod) -> Result<Option<Self>, syn::Error> {
        let mut guard_type = None;
        let mut args = None;
        let mut err = None;
        impl_item.attrs.retain(|attr| {
            match attr.parse_meta() {
                Ok(syn::Meta::List(list)) => {
                    if let Some(ident) = list.path.get_ident() {
                        if let Ok(gt) = GuardType::from_str(&*ident.to_string()) {
                            guard_type = Some(gt);
                            match Args::new(list.nested.into_iter().collect()) {
                                Ok(ar) => {
                                    args = Some(ar);
                                }
                                Err(e) => err = Some(e),
                            }
                            return false;
                        }
                    }
                }
                Ok(syn::Meta::Path(path)) => {
                    if let Some(ident) = path.get_ident() {
                        if let Ok(gt) = GuardType::from_str(&*ident.to_string()) {
                            guard_type = Some(gt);
                            return false;
                        }
                    }
                }
                Ok(_) => {}
                Err(_) => {}
            }
            true
        });

        if let Some(err_inner) = err {
            return Err(err_inner);
        }

        match guard_type {
            Some(gt) => Ok(Some(Self {
                name: format_ident!("{}_{}", "__CONTRABAND_", impl_item.sig.ident),
                guard_type: gt,
                args: args.unwrap_or_default(),
                impl_item: impl_item.clone(),
            })),
            None => Ok(None),
        }
    }
}

impl ToTokens for Method {
    fn to_tokens(&self, stream: &mut TokenStream2) {
        let Self {
            name,
            guard_type,
            args:
                Args {
                    path,
                    guards,
                    wrappers,
                },
            impl_item,
        } = self;
        let target = &impl_item.sig.ident;
        let expanded = quote! {
            #[allow(non_snake_case)]
            fn #name(&self) -> actix_web::Resource {
                actix_web::web::resource(#path)
                    .guard(actix_web::guard::#guard_type())
                    #(.guard(actix_web::guard::fn_guard(#guards)))*
                    #(.wrap(#wrappers))*
                    .to(Self::#target)
            }
        };
        stream.extend(expanded)
    }
}

/// Creates a controller.
///
/// Syntax: `#[controller("path")]`
///
/// ## Example
///
/// ```rust,no_run
/// use contraband::{Injectable, controller};
/// use contraband::core::ContrabandApp;
/// use actix_web::HttpResponse;
///
/// #[derive(Clone, Injectable)]
/// struct HelloController;
///
/// #[controller]
/// impl HelloController {
///     #[get]
///     async fn hello_world(self) -> HttpResponse {
///         HttpResponse::Ok().body("Hello world!")
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn controller(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as ItemImpl);
    let mut methods = Vec::new();
    for item in &mut input.items {
        if let syn::ImplItem::Method(ref mut item_method) = item {
            match Method::new(item_method) {
                Ok(Some(method)) => {
                    methods.push(method);
                }
                Ok(None) => {}
                Err(err) => {
                    return err.to_compile_error().into();
                }
            }
        }
    }

    match args::Args::new(parse_macro_input!(attr as syn::AttributeArgs)) {
        Ok(args::Args {
            path,
            guards,
            wrappers,
        }) => {
            let route_idents: Vec<&syn::Ident> = methods.iter().map(|x| &x.name).collect();
            let name = &input.self_ty;
            let expanded = quote! {
                #input
                impl #name {
                    #(#methods)*
                }

                #[automatically_derived]
                impl actix_web::FromRequest for #name {
                    type Error = actix_web::Error;
                    type Future = futures_util::future::Ready<Result<Self, Self::Error>>;
                    type Config = ();

                    #[inline]
                    fn from_request(req: &actix_web::HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
                        match req.app_data::<actix_web::web::Data<#name>>() {
                            Some(st) => futures_util::future::ok(st.get_ref().clone()),
                            None => panic!("Failed to extract data class."),
                        }
                    }
                }

                #[automatically_derived]
                impl contraband::module::ServiceFactory for #name {
                    fn register(&self, app: &mut actix_web::web::ServiceConfig) {
                        app.service(
                            actix_web::web::scope(#path)
                            .data(self.clone())
                            #(.guard(actix_web::guard::fn_guard(#guards)))*
                            #(.wrap(#wrappers))*
                            #(.service(Self::#route_idents(&self)))*
                        );
                    }
                }
            };
            TokenStream::from(expanded)
        }
        Err(err) => err.to_compile_error().into(),
    }
}
