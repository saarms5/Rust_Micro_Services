//! Telemetry and monitoring module
//! 
//! This crate handles logging, metrics, and observability
//! for the microservices application.

pub mod logger;
pub mod metrics;

pub use logger::{Logger, LogLevel};
pub use metrics::Metrics;
