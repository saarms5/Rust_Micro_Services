//! Telemetry and monitoring module
//! 
//! This crate handles logging, metrics, observability, and telemetry schema
//! for the microservices application.

pub mod logger;
pub mod metrics;
pub mod types;
pub mod collector;
pub mod transports;

pub use logger::{Logger, LogLevel};
pub use metrics::Metrics;
pub use types::{
    SystemHealth, HealthStatus, SensorData, SensorReading, DiagnosticEntry, DiagnosticLevel,
    DiagnosticsReport, TelemetryPacket, ComponentId, Timestamp,
};
pub use collector::TelemetryCollector;
