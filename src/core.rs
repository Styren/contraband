extern crate rustls;

use actix_tls::rustls::ServerConfig as RustlsServerConfig;

use super::graph::Graph;
use crate::log::{ConsoleLoggingProvider, LogLevel, Logger, LoggingProvider};
use crate::module::{Context, ModuleFactory, ResolvedModule};
use actix_web::{dev::Service, App, HttpServer};
use listenfd::ListenFd;
use std::collections::HashMap;
use std::sync::Arc;

struct AppConfig {
    pub port: u16,
    pub logging_provider: Arc<dyn LoggingProvider>,
    pub log_level: LogLevel,
    pub tls_config: Option<RustlsServerConfig>,
}

impl AppConfig {
    fn new() -> Self {
        Self {
            port: 3000,
            logging_provider: Arc::new(ConsoleLoggingProvider),
            log_level: LogLevel::Info,
            tls_config: None,
        }
    }

    fn register_global_providers(&mut self) -> Context {
        let mut graph = Graph::new();
        graph.provide(Arc::new(Logger::new(
            self.logging_provider.clone(),
            self.log_level,
        )));
        Context {
            global_providers: graph,
            modules: HashMap::new(),
        }
    }
}

pub struct ContrabandApp {
    app_config: AppConfig,
}

impl Default for ContrabandApp {
    fn default() -> Self {
        Self::new()
    }
}

impl ContrabandApp {
    pub fn new() -> Self {
        Self {
            app_config: AppConfig::new(),
        }
    }

    pub fn load_config(_: HashMap<Option<String>, toml::Value>) {}

    fn configure(module: Arc<ResolvedModule>, cfg: &mut actix_web::web::ServiceConfig) {
        for controller in &module.controllers {
            controller.register(cfg);
        }
        for imported_module in &module.imported_modules {
            Self::configure(imported_module.clone(), cfg);
        }
    }

    /// Sets the logging provider of the application. This provider will be used when the [`Logger`] is
    /// injected.
    pub fn set_logging_provider<T: LoggingProvider + 'static>(
        mut self,
        logging_provider: T,
    ) -> Self {
        self.app_config.logging_provider = Arc::new(logging_provider);
        self
    }

    /// Sets the loglevel for the application. No messages will be logged for any severity level
    /// below the provided [`LogLevel`].
    pub fn set_loglevel(mut self, log_level: LogLevel) -> Self {
        self.app_config.log_level = log_level;
        self
    }

    pub fn set_port(mut self, port: u16) -> Self {
        self.app_config.port = port;
        self
    }

    #[cfg(feature = "rustls")]
    pub fn set_tls_config(mut self, tls_config: RustlsServerConfig) -> Self {
        self.app_config.tls_config = Some(tls_config);
        self
    }

    pub async fn start<T: ModuleFactory>(mut self) -> std::io::Result<()> {
        let mut listenfd = ListenFd::from_env();
        let mut ctx: Context = self.app_config.register_global_providers();
        let module = Arc::new(T::get_module().build(&mut ctx));
        let mut server = HttpServer::new(move || {
            App::new().configure(|cfg| Self::configure(module.clone(), cfg))
        });

        if cfg!(feature = "rustls") && self.app_config.tls_config.is_some() {
            server = if let Some(l) = listenfd.take_tcp_listener(0).unwrap() {
                server.listen_rustls(l, self.app_config.tls_config.unwrap())?
            } else {
                server.bind_rustls(
                    format!("0.0.0.0:{}", self.app_config.port),
                    self.app_config.tls_config.unwrap(),
                )?
            }
        } else {
            server = if let Some(l) = listenfd.take_tcp_listener(0).unwrap() {
                server.listen(l)?
            } else {
                server.bind(format!("0.0.0.0:{}", self.app_config.port))?
            }
        };

        server.run().await
    }

    pub async fn test_server<T: ModuleFactory>(
        mut self,
    ) -> impl Service<
        Response = actix_web::dev::ServiceResponse,
        Request = actix_http::Request,
        Error = actix_web::Error,
    > {
        use actix_web::test;

        let mut ctx: Context = self.app_config.register_global_providers();
        let module = Arc::new(T::get_module().build(&mut ctx));
        test::init_service(App::new().configure(|cfg| Self::configure(module.clone(), cfg))).await
    }
}
