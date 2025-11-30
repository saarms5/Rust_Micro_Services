# Simulation API Documentation

## Overview

The **Simulation API** is a clean, well-defined interface that allows external simulation teams to interact with the embedded Rust microservices framework. It provides comprehensive control over components, sensors, actuators, and telemetry collection.

## Table of Contents

- [Quick Start](#quick-start)
- [Core Types](#core-types)
- [API Functions](#api-functions)
- [Example Usage](#example-usage)
- [Integration Guide](#integration-guide)
- [Error Handling](#error-handling)

---

## Quick Start

### Installation

Add the `app` crate to your `Cargo.toml`:

```toml
[dependencies]
app = { path = "../crates/app" }
tokio = { version = "1", features = ["full"] }
```

### Minimal Example

```rust
use app::{SimulationEngine, SimulationConfig, SensorData};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create simulation engine
    let engine = SimulationEngine::new(SimulationConfig::default()).await?;

    // Register a temperature sensor
    engine.register_sensor("temp-1", "TemperatureSensor").await?;

    // Inject sensor data
    engine
        .inject_sensor_data("temp-1", SensorData::temperature(25.5))
        .await?;

    // Execute control loop iteration
    engine.execute_iteration().await?;

    // Collect telemetry
    let telemetry = engine.collect_telemetry().await?;
    println!("Iteration {}: collected {} components", 
             telemetry.sequence, 
             telemetry.component_stats.len());

    // Shutdown
    engine.shutdown().await?;

    Ok(())
}
```

---

## Core Types

### `SimulationConfig`

Configuration for the simulation engine.

```rust
pub struct SimulationConfig {
    pub name: String,              // Simulation identifier
    pub control_loop_hz: u32,      // Control loop frequency (default: 50 Hz)
    pub enable_telemetry: bool,    // Enable telemetry collection (default: true)
    pub enable_realtime: bool,     // Enable real-time scheduling (default: false)
    pub timeout_secs: u64,         // Operation timeout (default: 30s)
}
```

**Default values:**
```rust
SimulationConfig {
    name: "default_simulation",
    control_loop_hz: 50,
    enable_telemetry: true,
    enable_realtime: false,
    timeout_secs: 30,
}
```

### `SensorData`

Enum representing different types of sensor data that can be injected.

```rust
pub enum SensorData {
    Temperature(f64),                    // Celsius
    Pressure(f64),                       // hPa
    GpsPosition(f64, f64, f64),         // lat, lon, alt
    Acceleration(f64, f64, f64),        // x, y, z in m/s²
    Numeric(f64),                        // Generic numeric value
    String(String),                      // Generic string
    Bool(bool),                          // Boolean state
}
```

**Helper constructors:**
```rust
SensorData::temperature(25.5)
SensorData::pressure(1013.25)
SensorData::gps_position(37.7749, -122.4194, 52.0)
SensorData::acceleration(9.8, 0.0, 0.0)
SensorData::numeric(42.0)
SensorData::string("event_triggered".to_string())
SensorData::bool(true)
```

### `ActuatorCommand`

Commands that can be sent to actuators.

```rust
pub enum ActuatorCommand {
    MotorSpeed(f64),        // 0.0 to 1.0
    Toggle(bool),           // on/off
    Position(f64),          // 0.0 to 1.0
    Custom(String),         // Custom command string
}
```

### `TelemetrySnapshot`

Snapshot of current system state and metrics.

```rust
pub struct TelemetrySnapshot {
    pub health: Option<SystemHealth>,
    pub component_stats: HashMap<String, ComponentStats>,
    pub sequence: u64,
    pub timestamp: String,  // ISO 8601 format
}
```

### `ComponentStats`

Statistics for a single component.

```rust
pub struct ComponentStats {
    pub id: String,
    pub name: String,
    pub iterations: u64,
    pub errors: u64,
    pub last_update: String,
}
```

---

## API Functions

### Initialization & Lifecycle

#### `SimulationEngine::new(config: SimulationConfig) -> Result<SimulationEngine>`

Create a new simulation engine.

```rust
let config = SimulationConfig {
    name: "my_simulation".to_string(),
    control_loop_hz: 100,
    ..Default::default()
};
let engine = SimulationEngine::new(config).await?;
```

#### `initialize_all() -> Result<()>`

Initialize all registered components.

```rust
engine.initialize_all().await?;
```

#### `shutdown() -> Result<()>`

Graceful shutdown - cleanup resources and stop loops.

```rust
engine.shutdown().await?;
```

### Component Registration

#### `register_sensor(id: &str, sensor_type: &str) -> Result<()>`

Register a sensor component.

```rust
engine.register_sensor("temp-sensor-1", "TemperatureSensor").await?;
engine.register_sensor("gps-1", "GpsSensor").await?;
```

#### `register_actuator(id: &str, actuator_type: &str) -> Result<()>`

Register an actuator component.

```rust
engine.register_actuator("motor-1", "Motor").await?;
engine.register_actuator("servo-1", "Servo").await?;
```

#### `list_sensors() -> Result<Vec<(String, String)>>`

Get list of all registered sensors.

```rust
let sensors = engine.list_sensors().await?;
for (id, sensor_type) in sensors {
    println!("{}: {}", id, sensor_type);
}
```

#### `list_actuators() -> Result<Vec<(String, String)>>`

Get list of all registered actuators.

```rust
let actuators = engine.list_actuators().await?;
for (id, actuator_type) in actuators {
    println!("{}: {}", id, actuator_type);
}
```

### Data Injection & Control

#### `inject_sensor_data(sensor_id: &str, data: SensorData) -> Result<()>`

Inject sensor data into the system.

```rust
engine
    .inject_sensor_data("temp-sensor-1", SensorData::temperature(25.5))
    .await?;

engine
    .inject_sensor_data("gps-1", SensorData::gps_position(37.7749, -122.4194, 52.0))
    .await?;
```

#### `get_sensor_data(sensor_id: &str) -> Result<Option<SensorData>>`

Retrieve the latest data from a sensor.

```rust
if let Some(data) = engine.get_sensor_data("temp-sensor-1").await? {
    println!("Sensor data: {:?}", data);
}
```

#### `send_actuator_command(actuator_id: &str, command: ActuatorCommand) -> Result<()>`

Send a command to an actuator.

```rust
engine
    .send_actuator_command("motor-1", ActuatorCommand::MotorSpeed(0.5))
    .await?;

engine
    .send_actuator_command("servo-1", ActuatorCommand::Position(0.75))
    .await?;
```

#### `get_actuator_command(actuator_id: &str) -> Result<Option<ActuatorCommand>>`

Get the last command sent to an actuator.

```rust
if let Some(cmd) = engine.get_actuator_command("motor-1").await? {
    println!("Last command: {:?}", cmd);
}
```

### Execution & Telemetry

#### `execute_iteration() -> Result<()>`

Execute a single control loop iteration.

```rust
engine.execute_iteration().await?;
```

#### `run_simulation(iterations: u32) -> Result<()>`

Run the simulation for a specified number of iterations.

```rust
engine.run_simulation(100).await?;  // Run 100 iterations
```

#### `collect_telemetry() -> Result<TelemetrySnapshot>`

Collect current telemetry snapshot.

```rust
let telemetry = engine.collect_telemetry().await?;
println!("Sequence: {}", telemetry.sequence);
println!("Timestamp: {}", telemetry.timestamp);
println!("Components: {:?}", telemetry.component_stats.keys());
```

#### `get_health_status() -> Result<Option<SystemHealth>>`

Get current system health status.

```rust
if let Some(health) = engine.get_health_status().await? {
    println!("System health: {:?}", health);
}
```

#### `health_check_all() -> Result<()>`

Perform health check on all components.

```rust
engine.health_check_all().await?;
println!("All components healthy!");
```

### Utilities

#### `config() -> &SimulationConfig`

Get simulation configuration.

```rust
let cfg = engine.config();
println!("Control loop: {} Hz", cfg.control_loop_hz);
```

#### `get_iteration_count() -> u64`

Get the current iteration count.

```rust
let count = engine.get_iteration_count().await;
println!("Completed {} iterations", count);
```

---

## Example Usage

### Complete Simulation Scenario

```rust
use app::{SimulationEngine, SimulationConfig, SensorData, ActuatorCommand};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Create engine with custom config
    let config = SimulationConfig {
        name: "thermal_control_sim".to_string(),
        control_loop_hz: 100,
        enable_telemetry: true,
        enable_realtime: false,
        timeout_secs: 60,
    };
    let engine = SimulationEngine::new(config).await?;

    // 2. Register components
    engine.register_sensor("temp-sensor", "TemperatureSensor").await?;
    engine.register_actuator("heater", "Heater").await?;
    engine.register_actuator("fan", "Fan").await?;

    // 3. Initialize
    engine.initialize_all().await?;
    engine.health_check_all().await?;

    // 4. Run simulation loop
    for iteration in 0..50 {
        // Inject simulated temperature data
        let temp = 20.0 + (iteration as f64 * 0.5);
        engine
            .inject_sensor_data("temp-sensor", SensorData::temperature(temp))
            .await?;

        // Control logic: send commands based on temperature
        if temp > 30.0 {
            engine
                .send_actuator_command("heater", ActuatorCommand::MotorSpeed(0.0))
                .await?;
            engine
                .send_actuator_command("fan", ActuatorCommand::MotorSpeed(1.0))
                .await?;
        } else if temp < 20.0 {
            engine
                .send_actuator_command("heater", ActuatorCommand::MotorSpeed(1.0))
                .await?;
            engine
                .send_actuator_command("fan", ActuatorCommand::MotorSpeed(0.0))
                .await?;
        }

        // Execute iteration
        engine.execute_iteration().await?;

        // Collect telemetry
        let telemetry = engine.collect_telemetry().await?;
        println!(
            "Iteration {}: {} components, timestamp: {}",
            telemetry.sequence,
            telemetry.component_stats.len(),
            telemetry.timestamp
        );

        // Small delay for simulation timing
        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    // 5. Shutdown
    engine.shutdown().await?;
    println!("Simulation completed!");

    Ok(())
}
```

### Multi-Sensor Scenario

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let engine = SimulationEngine::new(SimulationConfig::default()).await?;

    // Register multiple sensors
    let sensors = vec![
        ("temp-1", "TemperatureSensor"),
        ("pressure-1", "PressureSensor"),
        ("gps-1", "GpsSensor"),
        ("imu-1", "IMUSensor"),
    ];

    for (id, sensor_type) in sensors {
        engine.register_sensor(id, sensor_type).await?;
    }

    engine.initialize_all().await?;

    // Inject different types of data
    engine
        .inject_sensor_data("temp-1", SensorData::temperature(25.5))
        .await?;
    engine
        .inject_sensor_data("pressure-1", SensorData::pressure(1013.25))
        .await?;
    engine
        .inject_sensor_data("gps-1", SensorData::gps_position(37.7749, -122.4194, 10.0))
        .await?;
    engine
        .inject_sensor_data(
            "imu-1",
            SensorData::acceleration(9.8, 0.0, 0.0),
        )
        .await?;

    // Execute and collect telemetry
    engine.execute_iteration().await?;
    let telemetry = engine.collect_telemetry().await?;

    println!("Collected {} sensors:", telemetry.component_stats.len());
    for (id, stats) in &telemetry.component_stats {
        println!("  {}: {} iterations", id, stats.iterations);
    }

    engine.shutdown().await?;
    Ok(())
}
```

---

## Integration Guide

### For Simulation Teams

1. **Add Dependency**: Include `app` crate in your test/simulation crate's `Cargo.toml`

2. **Create Engine**: Instantiate `SimulationEngine` with desired configuration

3. **Register Components**: Use `register_sensor()` and `register_actuator()` for each component

4. **Inject Data**: Use `inject_sensor_data()` to provide inputs at each iteration

5. **Execute Loop**: Call `execute_iteration()` or `run_simulation()` to run control logic

6. **Collect Results**: Use `collect_telemetry()` to get outputs and system state

7. **Shutdown**: Call `shutdown()` for cleanup

### Recommended Patterns

**Pattern 1: Scenario-Based Testing**
```rust
// Define scenarios with expected inputs/outputs
let scenarios = vec![
    // (sensor_data, expected_actuator_command)
    (SensorData::temperature(15.0), ActuatorCommand::MotorSpeed(1.0)),
    (SensorData::temperature(25.0), ActuatorCommand::MotorSpeed(0.5)),
    (SensorData::temperature(35.0), ActuatorCommand::MotorSpeed(0.0)),
];

for (data, expected) in scenarios {
    engine.inject_sensor_data("temp", data).await?;
    engine.execute_iteration().await?;
    let cmd = engine.get_actuator_command("motor").await?;
    assert_eq!(cmd, Some(expected));
}
```

**Pattern 2: Continuous Monitoring**
```rust
// Run simulation and monitor telemetry in real-time
for _ in 0..100 {
    engine.execute_iteration().await?;
    let telemetry = engine.collect_telemetry().await?;
    
    // Process telemetry
    if telemetry.health.is_some() {
        // Handle health data
    }
}
```

**Pattern 3: Fault Injection**
```rust
// Test error handling by injecting faults
engine
    .inject_sensor_data("sensor-1", SensorData::numeric(-999.0))  // Out of range
    .await?;

engine.execute_iteration().await?;

// Verify error handling
let stats = engine.collect_telemetry().await?;
if stats.component_stats["sensor-1"].errors > 0 {
    println!("Error handling verified!");
}
```

---

## Error Handling

All API functions return `Result<T>` for proper error handling.

```rust
use rms_core::ComponentError;

match engine.inject_sensor_data("unknown-sensor", SensorData::temperature(25.0)).await {
    Ok(_) => println!("Data injected successfully"),
    Err(ComponentError { message }) => {
        eprintln!("Failed to inject data: {}", message);
        // Handle error - e.g., sensor not registered
    }
}
```

**Common Errors:**
- `"Sensor {id} not found"` - Sensor not registered
- `"Actuator {id} not found"` - Actuator not registered
- Operation timeouts - Configure via `SimulationConfig::timeout_secs`

---

## API Reference Summary

| Function | Purpose |
|----------|---------|
| `new()` | Create engine instance |
| `register_sensor()` | Register sensor component |
| `register_actuator()` | Register actuator component |
| `initialize_all()` | Initialize all components |
| `inject_sensor_data()` | Inject sensor input |
| `send_actuator_command()` | Send actuator command |
| `execute_iteration()` | Run one control loop cycle |
| `run_simulation()` | Run multiple iterations |
| `collect_telemetry()` | Get system snapshot |
| `get_health_status()` | Check system health |
| `health_check_all()` | Perform health check |
| `shutdown()` | Cleanup and shutdown |
| `list_sensors()` | Get registered sensors |
| `list_actuators()` | Get registered actuators |
| `get_sensor_data()` | Retrieve sensor reading |
| `get_actuator_command()` | Get last actuator command |
| `config()` | Get configuration |
| `get_iteration_count()` | Get iteration counter |

---

## Support & Questions

For issues or questions about the Simulation API:
1. Check example code above
2. Review error messages (ComponentError has detailed descriptions)
3. Refer to specific function documentation in `app/src/simulation_api.rs`
