# Simulation API Quick Reference

## Import & Setup

```rust
use app::{SimulationEngine, SimulationConfig, SensorData, ActuatorCommand};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create engine
    let engine = SimulationEngine::new(SimulationConfig::default()).await?;
    
    // ... your code ...
    
    Ok(())
}
```

## Minimal Working Example (5 lines)

```rust
let engine = SimulationEngine::new(SimulationConfig::default()).await?;
engine.register_sensor("temp-1", "TemperatureSensor").await?;
engine.inject_sensor_data("temp-1", SensorData::temperature(25.5)).await?;
engine.execute_iteration().await?;
let telemetry = engine.collect_telemetry().await?;
```

---

## Sensor Data Types

```rust
SensorData::temperature(25.5)                           // Celsius
SensorData::pressure(1013.25)                           // hPa
SensorData::gps_position(37.7749, -122.4194, 52.0)     // lat, lon, alt
SensorData::acceleration(9.8, 0.0, 0.0)                // x, y, z (m/s²)
SensorData::numeric(42.0)                              // Any number
SensorData::string("event".to_string())                // Any string
SensorData::bool(true)                                 // Boolean
```

## Actuator Commands

```rust
ActuatorCommand::MotorSpeed(0.5)        // 0.0 = off, 1.0 = full speed
ActuatorCommand::Toggle(true)           // on/off
ActuatorCommand::Position(0.75)         // 0.0 to 1.0
ActuatorCommand::Custom("reset".to_string())  // Custom strings
```

---

## Core Operations

| Task | Code |
|------|------|
| Create engine | `SimulationEngine::new(config).await?` |
| Register sensor | `engine.register_sensor("id", "type").await?` |
| Register actuator | `engine.register_actuator("id", "type").await?` |
| Inject data | `engine.inject_sensor_data("id", data).await?` |
| Send command | `engine.send_actuator_command("id", cmd).await?` |
| Execute once | `engine.execute_iteration().await?` |
| Run N times | `engine.run_simulation(100).await?` |
| Get telemetry | `engine.collect_telemetry().await?` |
| Health check | `engine.health_check_all().await?` |
| List sensors | `engine.list_sensors().await?` |
| List actuators | `engine.list_actuators().await?` |
| Get sensor value | `engine.get_sensor_data("id").await?` |
| Get last command | `engine.get_actuator_command("id").await?` |
| Shutdown | `engine.shutdown().await?` |

---

## Configuration

```rust
let config = SimulationConfig {
    name: "my_sim".to_string(),
    control_loop_hz: 50,          // Default: 50
    enable_telemetry: true,       // Default: true
    enable_realtime: false,       // Default: false
    timeout_secs: 30,             // Default: 30
};

let engine = SimulationEngine::new(config).await?;
```

---

## Telemetry Output

```rust
let telemetry = engine.collect_telemetry().await?;

// Access telemetry data
println!("Iteration: {}", telemetry.sequence);
println!("Timestamp: {}", telemetry.timestamp);  // ISO 8601

for (id, stats) in &telemetry.component_stats {
    println!("Component {} - Iterations: {}, Errors: {}", 
             id, stats.iterations, stats.errors);
}

if let Some(health) = telemetry.health {
    println!("Health: {:?}", health);
}
```

---

## Error Handling

```rust
match engine.inject_sensor_data("temp-1", SensorData::temperature(25.0)).await {
    Ok(_) => println!("Success"),
    Err(e) => eprintln!("Error: {}", e.message),  // e.message contains details
}

// Or use ? operator for auto-propagation
engine.inject_sensor_data("temp-1", data).await?;
```

---

## Complete Scenario Template

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Create & Configure
    let engine = SimulationEngine::new(SimulationConfig {
        name: "my_test".to_string(),
        control_loop_hz: 100,
        ..Default::default()
    }).await?;

    // 2. Register Components
    engine.register_sensor("temp", "TemperatureSensor").await?;
    engine.register_actuator("motor", "Motor").await?;

    // 3. Initialize
    engine.initialize_all().await?;
    engine.health_check_all().await?;

    // 4. Run Simulation Loop
    for i in 0..50 {
        // Inject inputs
        let temp = 20.0 + (i as f64 * 0.5);
        engine.inject_sensor_data("temp", SensorData::temperature(temp)).await?;

        // Send commands
        if temp > 30.0 {
            engine.send_actuator_command("motor", ActuatorCommand::MotorSpeed(1.0)).await?;
        }

        // Execute
        engine.execute_iteration().await?;

        // Collect results
        let telemetry = engine.collect_telemetry().await?;
        println!("Iteration {}: {}", telemetry.sequence, telemetry.timestamp);
    }

    // 5. Shutdown
    engine.shutdown().await?;
    Ok(())
}
```

---

## Type Aliases (for common use)

```rust
// For convenience, consider these shortcuts in your code:
type SimEngine = SimulationEngine;
type SimConfig = SimulationConfig;
type SensingData = SensorData;
type Command = ActuatorCommand;
```

---

## Common Patterns

### Pattern 1: Monitor Sensor Over Time
```rust
for iter in 0..100 {
    if let Some(data) = engine.get_sensor_data("sensor-1").await? {
        println!("Iteration {}: {:?}", iter, data);
    }
}
```

### Pattern 2: Check Actuator Response
```rust
engine.send_actuator_command("motor-1", ActuatorCommand::MotorSpeed(0.5)).await?;
engine.execute_iteration().await?;

if let Some(cmd) = engine.get_actuator_command("motor-1").await? {
    match cmd {
        ActuatorCommand::MotorSpeed(speed) => println!("Motor speed: {}", speed),
        _ => println!("Command: {:?}", cmd),
    }
}
```

### Pattern 3: Batch Simulation Runs
```rust
let test_cases = vec![
    SensorData::temperature(10.0),
    SensorData::temperature(25.0),
    SensorData::temperature(40.0),
];

for (idx, data) in test_cases.iter().enumerate() {
    engine.inject_sensor_data("temp", data.clone()).await?;
    engine.execute_iteration().await?;
    let telemetry = engine.collect_telemetry().await?;
    println!("Test case {}: iter {}", idx, telemetry.sequence);
}
```

---

## Troubleshooting

| Issue | Solution |
|-------|----------|
| "Sensor X not found" | Use `engine.list_sensors().await?` to verify registration |
| "Actuator X not found" | Use `engine.list_actuators().await?` to verify registration |
| No telemetry data | Call `collect_telemetry()` AFTER `execute_iteration()` |
| Component not initialized | Call `engine.initialize_all().await?` first |
| Sensor data is None | Check if data was actually injected with `get_sensor_data()` |

---

## See Also

- Full API Documentation: `docs/SIMULATION_API.md`
- Source Code: `crates/app/src/simulation_api.rs`
- Example Test Suite: `crates/app/src/simulation_api.rs` (test module)
- Component Traits: `crates/core/src/component.rs`
