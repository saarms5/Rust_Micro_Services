//! Simulation API Library
//!
//! This library provides a clean interface for external simulation teams to interact
//! with the embedded Rust microservices framework.
//!
//! # Quick Start
//!
//! ```no_run
//! use app::simulation_api::{SimulationEngine, SimulationConfig, SensorData};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create and initialize engine
//!     let engine = SimulationEngine::new(SimulationConfig::default()).await?;
//!
//!     // Register components
//!     engine.register_sensor("temp-1", "TemperatureSensor").await?;
//!     engine.register_actuator("motor-1", "Motor").await?;
//!
//!     // Inject sensor data
//!     engine
//!         .inject_sensor_data("temp-1", SensorData::temperature(25.0))
//!         .await?;
//!
//!     // Execute control loop
//!     engine.execute_iteration().await?;
//!
//!     // Get telemetry
//!     let telemetry = engine.collect_telemetry().await?;
//!     println!("Iteration: {}", telemetry.sequence);
//!
//!     Ok(())
//! }
//! ```

pub mod simulation_api;

// Re-export commonly used types for convenience
pub use simulation_api::{
    ActuatorCommand, ComponentStats, SensorData, SimulationConfig, SimulationEngine,
    TelemetrySnapshot,
};
