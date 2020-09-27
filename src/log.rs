use crate::graph::Graph;
use futures_util::future::{ok, Ready};
use std::sync::Arc;

#[derive(Clone, Copy)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

pub trait LoggingProvider: Sync + Send {
    fn debug(&self, message: String);

    fn info(&self, message: String);

    fn warn(&self, message: String);

    fn error(&self, message: String);
}

#[derive(Clone)]
pub struct ConsoleLoggingProvider;

impl ConsoleLoggingProvider {
    #[inline]
    fn log(log_level: &str, message: String) {
        let now = chrono::Utc::now();
        println!("{}: {} {}", now, log_level, message);
    }
}

impl LoggingProvider for ConsoleLoggingProvider {
    fn debug(&self, message: String) {
        Self::log("DEBUG", message);
    }

    fn info(&self, message: String) {
        Self::log("INFO", message);
    }

    fn warn(&self, message: String) {
        Self::log("WARN", message);
    }

    fn error(&self, message: String) {
        Self::log("ERROR", message);
    }
}

impl crate::graph::Injected for Logger {
    type Output = Self;
    fn resolve(_: &mut crate::graph::Graph, _: &[&Graph]) -> Self {
        panic!("No logger provided.")
    }
}

#[derive(Clone)]
pub struct Logger {
    logging_provider: Arc<dyn LoggingProvider>,
    log_level: LogLevel,
}

impl Logger {
    pub fn new(logging_provider: Arc<dyn LoggingProvider>, log_level: LogLevel) -> Self {
        Self {
            logging_provider,
            log_level,
        }
    }

    #[inline]
    pub fn debug(&self, message: String) {
        if self.log_level as u64 <= LogLevel::Debug as u64 {
            self.logging_provider.debug(message);
        }
    }

    #[inline]
    pub fn info(&self, message: String) {
        if self.log_level as u64 <= LogLevel::Info as u64 {
            self.logging_provider.info(message);
        }
    }

    #[inline]
    pub fn warn(&self, message: String) {
        if self.log_level as u64 <= LogLevel::Warn as u64 {
            self.logging_provider.warn(message);
        }
    }

    #[inline]
    pub fn error(&self, message: String) {
        if self.log_level as u64 <= LogLevel::Error as u64 {
            self.logging_provider.error(message);
        }
    }
}

impl actix_web::FromRequest for Logger {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;
    type Config = ();
    #[inline]
    fn from_request(req: &actix_web::HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        match req.app_data::<actix_web::web::Data<Self>>() {
            Some(st) => ok(st.get_ref().clone()),
            None => panic!("Failed to extract logger."),
        }
    }
}
