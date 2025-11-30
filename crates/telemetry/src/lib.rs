//! Telemetry and monitoring module
//!
//! This crate handles logging, metrics, observability, and telemetry schema
//! for the microservices application.

pub mod collector;
pub mod config;
pub mod logger;
pub mod metrics;
pub mod resilience;
pub mod streaming;
pub mod transports;
pub mod types;

#[cfg(feature = "mqtt_real")]
pub mod mqtt_real;

pub use collector::TelemetryCollector;
pub use config::{ConfigError, ConfigLoader, TelemetryConfig};
pub use logger::{LogLevel, Logger};
pub use metrics::Metrics;
pub use resilience::{CircuitBreaker, CircuitState, OfflineBuffer, ResilienceConfig, RetryStrategy};
pub use streaming::{PipelineConfig, StreamingPipeline};
pub use transports::{MqttTransport, SerialTransport, Transport, TransportError};
pub use types::{
    ComponentId, DiagnosticEntry, DiagnosticLevel, DiagnosticsReport, HealthStatus, SensorData,
    SensorReading, SystemHealth, TelemetryPacket, Timestamp,
};

#[cfg(feature = "mqtt_real")]
pub use mqtt_real::{MqttConfig, MqttError, RealMqttTransport};
