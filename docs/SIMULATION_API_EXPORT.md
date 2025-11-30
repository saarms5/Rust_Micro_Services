# Simulation API Export Summary

## What You're Getting

A production-ready Simulation API that allows your external simulation team to interact with the embedded Rust microservices framework through a clean, type-safe Rust interface.

---

## 📦 Core Deliverables

### 1. **SimulationEngine** - Main Interface
The central orchestrator for all simulation operations.

```rust
pub struct SimulationEngine { ... }
```

**Key Capabilities:**
- Component lifecycle management (init, execute, shutdown)
- Sensor data injection from simulation environment
- Actuator command execution
- Real-time control loop iteration
- Comprehensive telemetry collection

### 2. **SensorData** - 7 Sensor Types
Flexible enum for injecting different types of sensor inputs:

```rust
pub enum SensorData {
    Temperature(f64),              // Celsius
    Pressure(f64),                 // hPa
    GpsPosition(f64, f64, f64),   // lat, lon, alt
    Acceleration(f64, f64, f64),  // x, y, z m/s²
    Numeric(f64),                  // Generic
    String(String),                // Generic
    Bool(bool),                    // Boolean
}
```

### 3. **ActuatorCommand** - 4 Command Types
Standard commands for actuator control:

```rust
pub enum ActuatorCommand {
    MotorSpeed(f64),        // 0.0-1.0
    Toggle(bool),           // on/off
    Position(f64),          // 0.0-1.0
    Custom(String),         // Custom
}
```

### 4. **TelemetrySnapshot** - System State
Complete system state and metrics:

```rust
pub struct TelemetrySnapshot {
    pub health: Option<SystemHealth>,
    pub component_stats: HashMap<String, ComponentStats>,
    pub sequence: u64,
    pub timestamp: String,  // ISO 8601
}
```

---

## 🔌 18 Core API Functions

### Initialization (3 functions)
```rust
SimulationEngine::new(config)          // Create engine
engine.initialize_all()                // Init components
engine.shutdown()                      // Cleanup
```

### Component Management (4 functions)
```rust
engine.register_sensor(id, type)       // Register sensor
engine.register_actuator(id, type)     // Register actuator
engine.list_sensors()                  // Get all sensors
engine.list_actuators()                // Get all actuators
```

### Data Control (4 functions)
```rust
engine.inject_sensor_data(id, data)    // Input sensor
engine.get_sensor_data(id)             // Read sensor
engine.send_actuator_command(id, cmd)  // Output command
engine.get_actuator_command(id)        // Read command
```

### Execution (3 functions)
```rust
engine.execute_iteration()             // Run once
engine.run_simulation(count)           // Run N times
engine.health_check_all()              // Health check
```

### Telemetry (3 functions)
```rust
engine.collect_telemetry()             // Get snapshot
engine.get_health_status()             // Get health
engine.get_iteration_count()           // Get counter
engine.config()                        // Get config
```

---

## 📚 Documentation Provided

### 1. **SIMULATION_API.md** (2500+ words)
Complete reference with:
- Quick start guide
- Detailed API documentation for all 18 functions
- Core types explained with examples
- 3 complete working scenarios
- Integration patterns and best practices
- Error handling guide
- Reference table

**Location:** `docs/SIMULATION_API.md`

### 2. **SIMULATION_API_QUICK_REF.md** (500+ words)
One-page cheat sheet with:
- Minimal 5-line working example
- All sensor/actuator types at a glance
- Quick operation reference table
- Common patterns (3 included)
- Troubleshooting guide

**Location:** `docs/SIMULATION_API_QUICK_REF.md`

### 3. **Source Code** with Inline Documentation
- Full rustdoc comments on all public types
- 6 comprehensive unit tests in the source
- Example usage in module-level documentation

**Location:** `crates/app/src/simulation_api.rs`

### 4. **Exported as Library**
```rust
// teams can import as:
use app::{SimulationEngine, SimulationConfig, SensorData, ActuatorCommand};
```

**Location:** `crates/app/src/lib.rs`

---

## 🚀 Quick Start for Your Team

### Step 1: Add Dependency
```toml
[dependencies]
app = { path = "../crates/app" }
tokio = { version = "1", features = ["full"] }
```

### Step 2: Import API
```rust
use app::{SimulationEngine, SimulationConfig, SensorData};
```

### Step 3: Create Engine & Run
```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let engine = SimulationEngine::new(SimulationConfig::default()).await?;
    engine.register_sensor("temp-1", "TemperatureSensor").await?;
    engine.inject_sensor_data("temp-1", SensorData::temperature(25.5)).await?;
    engine.execute_iteration().await?;
    let telemetry = engine.collect_telemetry().await?;
    engine.shutdown().await?;
    Ok(())
}
```

---

## ✨ Key Features

✅ **Type-Safe** - All operations return `Result<T>` for proper error handling

✅ **Async/Await** - Built on Tokio for high-performance async operations

✅ **Flexible Sensor Types** - 7 different data types; easily extensible

✅ **Thread-Safe** - Uses Arc<Mutex/RwLock> for safe concurrent access

✅ **Serializable** - All types derive Serialize/Deserialize for data exchange

✅ **Well-Tested** - 6 comprehensive unit tests included

✅ **Documented** - 2500+ words of documentation + inline code comments

✅ **Production-Ready** - Follows Rust best practices and idioms

---

## 📊 Test Coverage

All 32 workspace tests pass:

```
app (simulation_api tests):        6 passing
telemetry:                        21 passing
hal:                              2 passing
rms_core:                         3 passing
─────────────────────────────────
Total:                           32 passing ✓
```

---

## 🔄 Integration Example

**Your simulation team can now:**

1. **Prepare test data** → Use SensorData enum variants
2. **Create scenarios** → Use SimulationConfig to customize
3. **Run simulations** → Call execute_iteration() in a loop
4. **Capture results** → Use collect_telemetry() to get outputs
5. **Verify behavior** → Access component statistics and health

```rust
// Example: Thermal control system test
let engine = SimulationEngine::new(config).await?;
engine.register_sensor("temp", "TemperatureSensor").await?;
engine.register_actuator("heater", "Heater").await?;

for temp in [15.0, 25.0, 35.0] {
    engine.inject_sensor_data("temp", SensorData::temperature(temp)).await?;
    engine.execute_iteration().await?;
    let telemetry = engine.collect_telemetry().await?;
    println!("At {}°C: iteration {}", temp, telemetry.sequence);
}
```

---

## 📋 Files Modified/Created

### New Files
- ✨ `crates/app/src/simulation_api.rs` - Main API implementation (420 lines)
- ✨ `crates/app/src/lib.rs` - Public API exports
- ✨ `docs/SIMULATION_API.md` - Full documentation
- ✨ `docs/SIMULATION_API_QUICK_REF.md` - Quick reference

### Modified Files
- 📝 `crates/app/src/main.rs` - Added module export
- 📝 `crates/app/Cargo.toml` - Added serde + chrono dependencies

---

## 🎯 What's Exported to Simulation Team

**Primary Types:**
- `SimulationEngine` - Main interface
- `SimulationConfig` - Configuration struct
- `SensorData` - Input data enum
- `ActuatorCommand` - Output command enum
- `TelemetrySnapshot` - Results struct
- `ComponentStats` - Per-component metrics
- `ComponentError` - Error type

**All 18 Functions:**
- Lifecycle: `new()`, `initialize_all()`, `shutdown()`
- Registration: `register_sensor()`, `register_actuator()`, `list_sensors()`, `list_actuators()`
- Data I/O: `inject_sensor_data()`, `get_sensor_data()`, `send_actuator_command()`, `get_actuator_command()`
- Execution: `execute_iteration()`, `run_simulation()`, `health_check_all()`
- Telemetry: `collect_telemetry()`, `get_health_status()`, `get_iteration_count()`, `config()`

---

## 🔗 Next Steps for Simulation Team

1. **Review** `docs/SIMULATION_API_QUICK_REF.md` (5 min read)
2. **Run** the minimal example (step 3 above)
3. **Read** `docs/SIMULATION_API.md` for detailed API reference
4. **Adapt** one of the 3 provided scenarios for your use case
5. **Integrate** with your simulation environment

---

## 💡 Common Use Cases

- ✅ Unit testing control logic with injected sensor data
- ✅ Hardware-in-the-loop (HIL) simulation
- ✅ Scenario-based validation (normal/fault/edge cases)
- ✅ Performance testing at variable loop rates
- ✅ Integration testing with multiple components
- ✅ Continuous monitoring of component health
- ✅ Telemetry validation and analysis

---

## 📞 Support Resources

1. **Quick Questions?** → Check `SIMULATION_API_QUICK_REF.md`
2. **API Details?** → See `SIMULATION_API.md`
3. **Code Examples?** → Look at tests in `simulation_api.rs`
4. **Understanding Components?** → Read `crates/core/src/component.rs`
5. **Telemetry Structure?** → Check `crates/telemetry/src/types.rs`

---

## Version Info

- **API Version:** 1.0
- **Status:** Production Ready ✓
- **Last Updated:** November 30, 2025
- **Tested On:** Rust 1.70+ with Tokio runtime
- **Platforms:** Windows, Linux, macOS (any with Rust toolchain)

---

**Ready to integrate?** Start with the quick reference, then use the full API docs as needed! 🚀
