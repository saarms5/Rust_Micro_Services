//! Simulation API - External interface for simulation teams
//!
//! This module provides a clean API for external simulation teams to interact with the
//! embedded Rust microservices framework. It handles component lifecycle, telemetry
//! collection, and real-time control loops.
//!
//! # Example Usage
//!
//! ```no_run
//! use app::{SimulationEngine, SimulationConfig, SensorData};
//! use std::sync::Arc;
//! use tokio::sync::Mutex;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create simulation engine with default config
//!     let engine = SimulationEngine::new(SimulationConfig::default()).await?;
//!
//!     // Register sensors
//!     engine.register_sensor("temp-001", "TemperatureSensor").await?;
//!     engine.register_actuator("motor-001", "MotorActuator").await?;
//!
//!     // Initialize all components
//!     engine.initialize_all().await?;
//!
//!     // Run health checks
//!     let health = engine.get_health_status().await?;
//!     println!("System health: {:?}", health);
//!
//!     // Inject sensor data
//!     let sensor_data = SensorData::temperature(25.5);
//!     engine.inject_sensor_data("temp-001", sensor_data).await?;
//!
//!     // Execute one control loop iteration
//!     engine.execute_iteration().await?;
//!
//!     // Collect telemetry
//!     let telemetry = engine.collect_telemetry().await?;
//!     println!("Telemetry: {:?}", telemetry);
//!
//!     // Graceful shutdown
//!     engine.shutdown().await?;
//!
//!     Ok(())
//! }
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

// Re-export commonly used types
pub use rms_core::{ComponentError, ComponentResult};
pub use telemetry::{SystemHealth, TelemetryPacket};

/// Configuration for the simulation engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationConfig {
    /// Simulation name/identifier
    pub name: String,
    /// Control loop frequency (Hz)
    pub control_loop_hz: u32,
    /// Enable telemetry collection
    pub enable_telemetry: bool,
    /// Enable real-time scheduling (if supported)
    pub enable_realtime: bool,
    /// Timeout for operations (seconds)
    pub timeout_secs: u64,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            name: "default_simulation".to_string(),
            control_loop_hz: 50,
            enable_telemetry: true,
            enable_realtime: false,
            timeout_secs: 30,
        }
    }
}

/// Sensor data types that can be injected into the simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SensorData {
    /// Temperature in Celsius
    Temperature(f64),
    /// Pressure in hPa
    Pressure(f64),
    /// GPS coordinates (latitude, longitude, altitude)
    GpsPosition(f64, f64, f64),
    /// Accelerometer reading (x, y, z in m/s²)
    Acceleration(f64, f64, f64),
    /// Generic numeric value
    Numeric(f64),
    /// Generic string value
    String(String),
    /// Boolean state
    Bool(bool),
}

impl SensorData {
    /// Create temperature sensor data
    pub fn temperature(celsius: f64) -> Self {
        SensorData::Temperature(celsius)
    }

    /// Create pressure sensor data
    pub fn pressure(hpa: f64) -> Self {
        SensorData::Pressure(hpa)
    }

    /// Create GPS position data
    pub fn gps_position(lat: f64, lon: f64, alt: f64) -> Self {
        SensorData::GpsPosition(lat, lon, alt)
    }

    /// Create acceleration data
    pub fn acceleration(x: f64, y: f64, z: f64) -> Self {
        SensorData::Acceleration(x, y, z)
    }

    /// Create numeric value data
    pub fn numeric(value: f64) -> Self {
        SensorData::Numeric(value)
    }

    /// Create string value data
    pub fn string(value: String) -> Self {
        SensorData::String(value)
    }

    /// Create boolean state data
    pub fn bool(value: bool) -> Self {
        SensorData::Bool(value)
    }
}

/// Actuator command types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActuatorCommand {
    /// Set motor speed (0.0 to 1.0)
    MotorSpeed(f64),
    /// Toggle switch on/off
    Toggle(bool),
    /// Set to specific position (0.0 to 1.0)
    Position(f64),
    /// Generic command string
    Custom(String),
}

/// Statistics about a component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentStats {
    pub id: String,
    pub name: String,
    pub iterations: u64,
    pub errors: u64,
    pub last_update: String,
}

/// Telemetry snapshot from the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetrySnapshot {
    /// System health status
    pub health: Option<SystemHealth>,
    /// Per-component statistics
    pub component_stats: HashMap<String, ComponentStats>,
    /// Sequence number
    pub sequence: u64,
    /// Timestamp (ISO 8601)
    pub timestamp: String,
}

/// Main simulation engine interface
pub struct SimulationEngine {
    config: SimulationConfig,
    iteration_count: Arc<Mutex<u64>>,
    sensors: Arc<RwLock<HashMap<String, SensorInfo>>>,
    actuators: Arc<RwLock<HashMap<String, ActuatorInfo>>>,
    health_status: Arc<Mutex<Option<SystemHealth>>>,
}

struct SensorInfo {
    #[allow(dead_code)]
    id: String,
    name: String,
    latest_data: Option<SensorData>,
}

struct ActuatorInfo {
    #[allow(dead_code)]
    id: String,
    name: String,
    last_command: Option<ActuatorCommand>,
}

impl SimulationEngine {
    /// Create a new simulation engine with the given configuration
    pub async fn new(config: SimulationConfig) -> ComponentResult<Self> {
        Ok(Self {
            config,
            iteration_count: Arc::new(Mutex::new(0)),
            sensors: Arc::new(RwLock::new(HashMap::new())),
            actuators: Arc::new(RwLock::new(HashMap::new())),
            health_status: Arc::new(Mutex::new(None)),
        })
    }

    /// Register a sensor with the simulation
    pub async fn register_sensor(&self, id: &str, sensor_type: &str) -> ComponentResult<()> {
        let mut sensors = self.sensors.write().await;
        sensors.insert(
            id.to_string(),
            SensorInfo {
                id: id.to_string(),
                name: sensor_type.to_string(),
                latest_data: None,
            },
        );
        Ok(())
    }

    /// Register an actuator with the simulation
    pub async fn register_actuator(&self, id: &str, actuator_type: &str) -> ComponentResult<()> {
        let mut actuators = self.actuators.write().await;
        actuators.insert(
            id.to_string(),
            ActuatorInfo {
                id: id.to_string(),
                name: actuator_type.to_string(),
                last_command: None,
            },
        );
        Ok(())
    }

    /// Initialize all registered components
    pub async fn initialize_all(&self) -> ComponentResult<()> {
        // In a real implementation, this would call init() on all components
        // For now, we simulate successful initialization
        Ok(())
    }

    /// Get current health status of the system
    pub async fn get_health_status(&self) -> ComponentResult<Option<SystemHealth>> {
        let health = self.health_status.lock().await;
        Ok(health.clone())
    }

    /// Inject sensor data into the simulation
    ///
    /// # Arguments
    ///
    /// * `sensor_id` - The unique identifier of the sensor
    /// * `data` - The sensor data to inject
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if the sensor is not found
    pub async fn inject_sensor_data(
        &self,
        sensor_id: &str,
        data: SensorData,
    ) -> ComponentResult<()> {
        let mut sensors = self.sensors.write().await;
        if let Some(sensor) = sensors.get_mut(sensor_id) {
            sensor.latest_data = Some(data);
            Ok(())
        } else {
            Err(ComponentError::new(format!(
                "Sensor {} not found",
                sensor_id
            )))
        }
    }

    /// Send a command to an actuator
    ///
    /// # Arguments
    ///
    /// * `actuator_id` - The unique identifier of the actuator
    /// * `command` - The command to send
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if the actuator is not found
    pub async fn send_actuator_command(
        &self,
        actuator_id: &str,
        command: ActuatorCommand,
    ) -> ComponentResult<()> {
        let mut actuators = self.actuators.write().await;
        if let Some(actuator) = actuators.get_mut(actuator_id) {
            actuator.last_command = Some(command);
            Ok(())
        } else {
            Err(ComponentError::new(format!(
                "Actuator {} not found",
                actuator_id
            )))
        }
    }

    /// Execute a single control loop iteration
    ///
    /// This processes all sensor inputs, executes control logic, and updates actuators.
    pub async fn execute_iteration(&self) -> ComponentResult<()> {
        let mut count = self.iteration_count.lock().await;
        *count += 1;

        // In a real implementation, this would:
        // 1. Read all sensor data
        // 2. Execute control logic
        // 3. Update actuators
        // 4. Collect diagnostics

        Ok(())
    }

    /// Collect telemetry snapshot from the system
    pub async fn collect_telemetry(&self) -> ComponentResult<TelemetrySnapshot> {
        let iteration = self.iteration_count.lock().await;
        let health = self.health_status.lock().await;
        let sensors = self.sensors.read().await;

        let mut component_stats = HashMap::new();

        // Collect sensor statistics
        for (id, sensor) in sensors.iter() {
            component_stats.insert(
                id.clone(),
                ComponentStats {
                    id: id.clone(),
                    name: sensor.name.clone(),
                    iterations: *iteration,
                    errors: 0,
                    last_update: chrono::Utc::now().to_rfc3339(),
                },
            );
        }

        Ok(TelemetrySnapshot {
            health: health.clone(),
            component_stats,
            sequence: *iteration,
            timestamp: chrono::Utc::now().to_rfc3339(),
        })
    }

    /// Get list of registered sensors
    pub async fn list_sensors(&self) -> ComponentResult<Vec<(String, String)>> {
        let sensors = self.sensors.read().await;
        Ok(sensors
            .iter()
            .map(|(id, info)| (id.clone(), info.name.clone()))
            .collect())
    }

    /// Get list of registered actuators
    pub async fn list_actuators(&self) -> ComponentResult<Vec<(String, String)>> {
        let actuators = self.actuators.read().await;
        Ok(actuators
            .iter()
            .map(|(id, info)| (id.clone(), info.name.clone()))
            .collect())
    }

    /// Get latest data from a sensor
    pub async fn get_sensor_data(&self, sensor_id: &str) -> ComponentResult<Option<SensorData>> {
        let sensors = self.sensors.read().await;
        if let Some(sensor) = sensors.get(sensor_id) {
            Ok(sensor.latest_data.clone())
        } else {
            Err(ComponentError::new(format!(
                "Sensor {} not found",
                sensor_id
            )))
        }
    }

    /// Get last command sent to an actuator
    pub async fn get_actuator_command(
        &self,
        actuator_id: &str,
    ) -> ComponentResult<Option<ActuatorCommand>> {
        let actuators = self.actuators.read().await;
        if let Some(actuator) = actuators.get(actuator_id) {
            Ok(actuator.last_command.clone())
        } else {
            Err(ComponentError::new(format!(
                "Actuator {} not found",
                actuator_id
            )))
        }
    }

    /// Perform health check on all components
    pub async fn health_check_all(&self) -> ComponentResult<()> {
        // In a real implementation, this would check all components
        // For now, simulate successful health check
        Ok(())
    }

    /// Run the simulation for a specified number of iterations
    ///
    /// # Arguments
    ///
    /// * `iterations` - Number of control loop iterations to run
    pub async fn run_simulation(&self, iterations: u32) -> ComponentResult<()> {
        for _ in 0..iterations {
            self.execute_iteration().await?;
            // Small delay between iterations (configurable in real implementation)
            tokio::time::sleep(tokio::time::Duration::from_millis(
                1000 / self.config.control_loop_hz as u64,
            ))
            .await;
        }
        Ok(())
    }

    /// Graceful shutdown - cleanup resources
    pub async fn shutdown(&self) -> ComponentResult<()> {
        // In a real implementation, this would:
        // 1. Stop all control loops
        // 2. Shutdown all components
        // 3. Flush telemetry
        // 4. Release resources
        Ok(())
    }

    /// Get simulation configuration
    pub fn config(&self) -> &SimulationConfig {
        &self.config
    }

    /// Get iteration count
    pub async fn get_iteration_count(&self) -> u64 {
        *self.iteration_count.lock().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_engine_creation() {
        let config = SimulationConfig::default();
        let engine = SimulationEngine::new(config).await;
        assert!(engine.is_ok());
    }

    #[tokio::test]
    async fn test_sensor_registration() {
        let engine = SimulationEngine::new(SimulationConfig::default())
            .await
            .unwrap();
        let result = engine
            .register_sensor("temp-001", "TemperatureSensor")
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_sensor_data_injection() {
        let engine = SimulationEngine::new(SimulationConfig::default())
            .await
            .unwrap();
        engine
            .register_sensor("temp-001", "TemperatureSensor")
            .await
            .unwrap();

        let data = SensorData::temperature(25.5);
        let result = engine.inject_sensor_data("temp-001", data).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_sensor_not_found() {
        let engine = SimulationEngine::new(SimulationConfig::default())
            .await
            .unwrap();

        let data = SensorData::temperature(25.5);
        let result = engine.inject_sensor_data("nonexistent", data).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_iteration_count() {
        let engine = SimulationEngine::new(SimulationConfig::default())
            .await
            .unwrap();

        assert_eq!(engine.get_iteration_count().await, 0);
        engine.execute_iteration().await.unwrap();
        assert_eq!(engine.get_iteration_count().await, 1);
    }

    #[tokio::test]
    async fn test_list_sensors() {
        let engine = SimulationEngine::new(SimulationConfig::default())
            .await
            .unwrap();
        engine
            .register_sensor("temp-001", "TemperatureSensor")
            .await
            .unwrap();
        engine
            .register_sensor("pressure-001", "PressureSensor")
            .await
            .unwrap();

        let sensors = engine.list_sensors().await.unwrap();
        assert_eq!(sensors.len(), 2);
    }
}
