#![warn(
    clippy::use_self,
    clippy::cargo,
)]
//! Contraband is a web framework for building modular applications in Rust with dependency
//! injection and performant higher-level abstractions. It is build on top of
//! [`actix-web`](https://crates.io/crates/actix-web).
//!
//! Contraband is heavily inspired by Spring Boot and Nestjs.
//!
//! ## Example
//!
//! ```rust,no_run
//! use contraband::{Injectable, controller, module};
//! use contraband::core::ContrabandApp;
//! use actix_web::HttpResponse;
//!
//! #[derive(Clone, Injectable)]
//! struct HelloController;
//!
//! #[controller]
//! impl HelloController {
//!     #[get]
//!     async fn hello_world(self) -> HttpResponse {
//!         HttpResponse::Ok().body("Hello world!")
//!     }
//! }
//!
//! #[module]
//! #[controller(HelloController)]
//! struct HelloModule;
//!
//! #[contraband::main]
//! async fn main() -> std::io::Result<()> {
//!     ContrabandApp::new()
//!         .start::<HelloModule>()
//!         .await
//! }
//! ```
//!
//! ## Documentation & community resources
//!
//! * [GitHub repository](https://github.com/styren/contraband)
//! * [Examples](https://github.com/styren/contraband/tree/master/examples)
pub mod graph;
pub mod module;
pub mod core;
pub mod config;
pub mod log;

extern crate contraband_codegen;
extern crate actix_rt;

pub use contraband_codegen::*;
pub use actix_rt::System as Runtime;
