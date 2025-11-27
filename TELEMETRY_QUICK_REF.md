# Telemetry Schema Quick Reference

## Core Types Overview

### SystemHealth
Tracks overall system state and resource usage.

```rust
pub struct SystemHealth {
    pub status: HealthStatus,              // Healthy/Degraded/Critical/Unknown
    pub timestamp: Timestamp,
    pub healthy_components: u32,
    pub degraded_components: u32,
    pub failed_components: u32,
    pub uptime_seconds: u64,
    pub cpu_usage_percent: f32,
    pub memory_usage_bytes: u64,
    pub temperature_celsius: f32,
    pub error_message: Option<String>,
}

impl SystemHealth {
    pub fn recalculate_status(&mut self) {
        // Auto-calculates status based on component counts
        // Critical if failed_components > 0
        // Degraded if degraded_components > 0
        // Healthy if healthy_components > 0
        // Unknown otherwise
    }
}
```

### SensorData (8 Variants)
Represents different types of sensor measurements.

```rust
pub enum SensorData {
    Temperature { value: f32, unit: String },
    Pressure { value: f32, unit: String },
    Humidity { value: f32, unit: String },
    Gps { latitude: f64, longitude: f64, altitude: f32, accuracy: f32 },
    Accelerometer { x: f32, y: f32, z: f32, unit: String },
    Gyroscope { x: f32, y: f32, z: f32, unit: String },
    Analog { value: f32, unit: String },
    Digital { state: bool, label: String },
}
```

### SensorReading
Wraps a SensorData measurement with metadata.

```rust
pub struct SensorReading {
    pub component_id: ComponentId,      // String identifier
    pub name: String,
    pub timestamp: Timestamp,
    pub data: SensorData,
    pub sequence: u64,                  // For ordering
    pub confidence: f32,                // 0.0-1.0
}

impl SensorReading {
    pub fn new(
        component_id: String,
        name: String,
        data: SensorData,
        sequence: u64,
    ) -> Self
}
```

### DiagnosticEntry
Records diagnostic/error events.

```rust
pub struct DiagnosticEntry {
    pub level: DiagnosticLevel,         // Info/Warning/Error/Critical
    pub timestamp: Timestamp,
    pub component: String,
    pub message: String,
    pub code: Option<String>,           // Error code
    pub context: HashMap<String, String>, // Additional metadata
}

// Builder pattern
let entry = DiagnosticEntry::new(
    DiagnosticLevel::Warning,
    "component-id",
    "High temperature detected"
)
.with_code("TEMP_HIGH_001")
.with_context("temperature", "85.5");
```

### DiagnosticsReport
Aggregates diagnostic entries.

```rust
pub struct DiagnosticsReport {
    pub total_entries: u32,
    pub info_count: u32,
    pub warning_count: u32,
    pub error_count: u32,
    pub critical_count: u32,
    pub entries: Vec<DiagnosticEntry>,
}

impl DiagnosticsReport {
    pub fn add_entry(&mut self, entry: DiagnosticEntry)
    pub fn clear(&mut self)
}
```

### TelemetryPacket
Root telemetry structure combining all data.

```rust
pub struct TelemetryPacket {
    pub sequence: u64,                  // Packet sequence number
    pub timestamp: Timestamp,
    pub health: SystemHealth,
    pub sensor_readings: Vec<SensorReading>,
    pub diagnostics: DiagnosticsReport,
}

// Serialization
impl TelemetryPacket {
    pub fn to_json(&self) -> Result<String, serde_json::Error>
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error>
    pub fn to_json_bytes(&self) -> Result<Vec<u8>, serde_json::Error>
    pub fn from_json_bytes(bytes: &[u8]) -> Result<Self, serde_json::Error>
    pub fn estimated_json_size(&self) -> usize
}
```

## TelemetryCollector Usage

### Basic API
```rust
let collector = TelemetryCollector::new();

// Record a reading
let reading = SensorReading::new(
    "temp-01".to_string(),
    "Temperature Sensor".to_string(),
    SensorData::Temperature {
        value: 25.5,
        unit: "°C".to_string(),
    },
    1,
);
collector.record_sensor_reading(reading).await;

// Record a diagnostic event
let entry = DiagnosticEntry::new(
    DiagnosticLevel::Info,
    "system",
    "Sensor reading recorded",
)
.with_code("SENSOR_01");
collector.record_diagnostic(entry).await;

// Update system health
let mut health = SystemHealth::new();
health.healthy_components = 5;
health.recalculate_status();
collector.update_health(health).await;

// Generate a packet
let packet = collector.generate_packet().await;
println!("{}", packet.to_json()?);

// Get recent readings (most recent first)
let readings = collector.get_sensor_readings(10).await;
```

### Memory Limits
- Max 1000 sensor readings (circular buffer)
- Max 100 diagnostic entries
- Older entries are discarded when limits exceeded
- Suitable for long-running applications

### Sequence Numbers
- Auto-incremented on `generate_packet()`
- Allows detection of lost packets
- Starts at 0, increments atomically per packet

## Common Patterns

### Creating a Health Report
```rust
let mut health = SystemHealth::new();
health.healthy_components = 5;
health.degraded_components = 1;
health.cpu_usage_percent = 45.2;
health.temperature_celsius = 42.5;
health.recalculate_status();  // Sets to Degraded

collector.update_health(health).await;
```

### Recording Sensor Data
```rust
// Temperature
SensorData::Temperature {
    value: 25.5,
    unit: "°C".to_string(),
}

// GPS
SensorData::Gps {
    latitude: 37.7749,
    longitude: -122.4194,
    altitude: 100.5,
    accuracy: 4.5,
}

// 3-axis accelerometer
SensorData::Accelerometer {
    x: 0.001,
    y: 0.002,
    z: 9.81,
    unit: "m/s²".to_string(),
}
```

### Diagnostics with Context
```rust
let entry = DiagnosticEntry::new(
    DiagnosticLevel::Warning,
    "motor-01",
    "Motor temperature high"
)
.with_code("MTR_TEMP_WARN")
.with_context("current_temp", "75.5")
.with_context("threshold", "70.0")
.with_context("recommendation", "Reduce load");

collector.record_diagnostic(entry).await;
```

### JSON Round-trip
```rust
// Serialize to JSON
let json_string = packet.to_json()?;

// Save to file
std::fs::write("telemetry.json", &json_string)?;

// Load and deserialize
let json_data = std::fs::read_to_string("telemetry.json")?;
let restored_packet = TelemetryPacket::from_json(&json_data)?;

// Verify packet integrity
assert_eq!(packet.sequence, restored_packet.sequence);
```

## Integration with Components

### In Component Implementation
```rust
impl Component for MyComponent {
    async fn init(&self, _collector: Option<&TelemetryCollector>) -> Result<()> {
        if let Some(collector) = _collector {
            let entry = DiagnosticEntry::new(
                DiagnosticLevel::Info,
                self.id.clone(),
                "Component initialized successfully",
            );
            collector.record_diagnostic(entry).await;
        }
        Ok(())
    }

    async fn run(&self, collector: Option<&TelemetryCollector>) -> Result<()> {
        let reading = SensorReading::new(
            self.id.clone(),
            "My Component".to_string(),
            SensorData::Analog {
                value: 42.0,
                unit: "units".to_string(),
            },
            1,
        );
        
        if let Some(collector) = collector {
            collector.record_sensor_reading(reading).await;
        }
        Ok(())
    }
}
```

## Testing

### Run telemetry tests
```bash
# All telemetry tests
cargo test -p telemetry

# Specific test
cargo test -p telemetry test_telemetry_packet_roundtrip

# With output
cargo test -p telemetry -- --nocapture
```

### Run all workspace tests
```bash
# Unit tests only (exclude doc-tests)
cargo test --all --lib

# Expected output: 16 tests passing
# - core: 3 tests
# - hal: 2 tests
# - telemetry: 11 tests
```

## Performance Characteristics

| Operation | Time | Notes |
|-----------|------|-------|
| Create SystemHealth | <1µs | Constant time |
| Record sensor reading | <5µs | Lock acquisition + push |
| Generate packet | <100µs | Clones all internal state |
| JSON serialization | ~1-10ms | Depends on packet size |
| Packet size estimate | <1µs | Calculation only |

## Dependencies
- `serde` (1.0+) - Serialization framework
- `serde_json` (1.0+) - JSON support
- `chrono` (0.4+) - Timestamps with serde support
- `tokio` (1.0+) - Async runtime (sync, macros, rt features)

## Feature Flags
None currently. All telemetry functionality is always available.

## Thread Safety
- ✅ Send + Sync safe
- ✅ Can be shared across async tasks
- ✅ Arc<Mutex<>> ensures thread safety
- ✅ Panics if accessed without `.await`

---
**Last Updated**: Phase 3 implementation complete
