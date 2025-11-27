//! Telemetry data types for system health, sensor readings, and diagnostics
//!
//! All types implement Serde for JSON and binary serialization.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Timestamp type for telemetry events
pub type Timestamp = chrono::DateTime<chrono::Utc>;

/// Unique identifier for telemetry sources
pub type ComponentId = String;

/// System health status enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum HealthStatus {
    /// System is operating normally
    Healthy,
    /// System is degraded but operational
    Degraded,
    /// System has encountered a critical error
    Critical,
    /// System status is unknown
    Unknown,
}

impl std::fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Healthy => write!(f, "Healthy"),
            Self::Degraded => write!(f, "Degraded"),
            Self::Critical => write!(f, "Critical"),
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}

/// System-wide health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealth {
    /// Overall system status
    pub status: HealthStatus,
    /// Timestamp when this health report was generated
    pub timestamp: Timestamp,
    /// Number of healthy components
    pub healthy_components: u32,
    /// Number of degraded components
    pub degraded_components: u32,
    /// Number of failed components
    pub failed_components: u32,
    /// System uptime in seconds
    pub uptime_seconds: u64,
    /// CPU usage percentage (0-100)
    pub cpu_usage_percent: f32,
    /// Memory usage in bytes
    pub memory_usage_bytes: u64,
    /// Temperature in Celsius
    pub temperature_celsius: f32,
    /// Optional error message if status is not Healthy
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}

impl SystemHealth {
    /// Create a new system health report
    pub fn new() -> Self {
        Self {
            status: HealthStatus::Unknown,
            timestamp: chrono::Utc::now(),
            healthy_components: 0,
            degraded_components: 0,
            failed_components: 0,
            uptime_seconds: 0,
            cpu_usage_percent: 0.0,
            memory_usage_bytes: 0,
            temperature_celsius: 25.0,
            error_message: None,
        }
    }

    /// Calculate overall health status based on component counts
    pub fn recalculate_status(&mut self) {
        self.status = if self.failed_components > 0 {
            HealthStatus::Critical
        } else if self.degraded_components > 0 {
            HealthStatus::Degraded
        } else if self.healthy_components > 0 {
            HealthStatus::Healthy
        } else {
            HealthStatus::Unknown
        };
    }
}

impl Default for SystemHealth {
    fn default() -> Self {
        Self::new()
    }
}

/// Sensor data types enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SensorData {
    /// Temperature reading in Celsius
    Temperature { value: f32, unit: String },
    /// Pressure reading in hPa
    Pressure { value: f32, unit: String },
    /// Humidity reading as percentage (0-100)
    Humidity { value: f32, unit: String },
    /// GPS coordinates
    Gps {
        latitude: f64,
        longitude: f64,
        altitude: f32,
        accuracy: f32,
    },
    /// Accelerometer reading (3-axis)
    Accelerometer {
        x: f32,
        y: f32,
        z: f32,
        unit: String,
    },
    /// Gyroscope reading (3-axis)
    Gyroscope {
        x: f32,
        y: f32,
        z: f32,
        unit: String,
    },
    /// Generic analog value
    Analog { value: f32, unit: String },
    /// Generic digital state
    Digital { state: bool, label: String },
}

impl SensorData {
    /// Get a human-readable description of the sensor reading
    pub fn description(&self) -> String {
        match self {
            Self::Temperature { value, unit } => format!("Temperature: {:.1}{}", value, unit),
            Self::Pressure { value, unit } => format!("Pressure: {:.2}{}", value, unit),
            Self::Humidity { value, unit } => format!("Humidity: {:.1}{}", value, unit),
            Self::Gps {
                latitude,
                longitude,
                altitude,
                accuracy,
            } => format!(
                "GPS: ({:.4}, {:.4}) alt={:.1}m acc={:.1}m",
                latitude, longitude, altitude, accuracy
            ),
            Self::Accelerometer { x, y, z, unit } => {
                format!("Accel: [{:.2}, {:.2}, {:.2}]{}", x, y, z, unit)
            }
            Self::Gyroscope { x, y, z, unit } => {
                format!("Gyro: [{:.2}, {:.2}, {:.2}]{}", x, y, z, unit)
            }
            Self::Analog { value, unit } => format!("Analog: {:.2}{}", value, unit),
            Self::Digital { state, label } => {
                format!("{}: {}", label, if *state { "ON" } else { "OFF" })
            }
        }
    }
}

/// A single sensor reading with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorReading {
    /// Unique component ID
    pub component_id: ComponentId,
    /// Human-readable component name
    pub component_name: String,
    /// Timestamp when measurement was taken
    pub timestamp: Timestamp,
    /// The actual sensor data
    pub data: SensorData,
    /// Sequence number for ordering
    pub sequence: u64,
    /// Confidence level (0-100)
    pub confidence: f32,
}

impl SensorReading {
    /// Create a new sensor reading
    pub fn new(
        component_id: ComponentId,
        component_name: String,
        data: SensorData,
        sequence: u64,
    ) -> Self {
        Self {
            component_id,
            component_name,
            timestamp: chrono::Utc::now(),
            data,
            sequence,
            confidence: 95.0,
        }
    }
}

/// Diagnostic event types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum DiagnosticLevel {
    /// Informational message
    Info,
    /// Warning that something may need attention
    Warning,
    /// Error that degraded functionality
    Error,
    /// Critical error that stopped functionality
    Critical,
}

impl std::fmt::Display for DiagnosticLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Info => write!(f, "Info"),
            Self::Warning => write!(f, "Warning"),
            Self::Error => write!(f, "Error"),
            Self::Critical => write!(f, "Critical"),
        }
    }
}

/// Diagnostic report entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticEntry {
    /// Severity level of the diagnostic
    pub level: DiagnosticLevel,
    /// Timestamp of the diagnostic event
    pub timestamp: Timestamp,
    /// Component that triggered the diagnostic
    pub component_id: ComponentId,
    /// Diagnostic message
    pub message: String,
    /// Optional diagnostic code
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    /// Additional context data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<HashMap<String, String>>,
}

impl DiagnosticEntry {
    /// Create a new diagnostic entry
    pub fn new(
        level: DiagnosticLevel,
        component_id: ComponentId,
        message: impl Into<String>,
    ) -> Self {
        Self {
            level,
            timestamp: chrono::Utc::now(),
            component_id,
            message: message.into(),
            code: None,
            context: None,
        }
    }

    /// Add a diagnostic code
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }

    /// Add context data
    pub fn with_context(mut self, key: String, value: String) -> Self {
        if self.context.is_none() {
            self.context = Some(HashMap::new());
        }
        if let Some(ref mut ctx) = self.context {
            ctx.insert(key, value);
        }
        self
    }
}

/// Complete diagnostics report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticsReport {
    /// Timestamp of report generation
    pub timestamp: Timestamp,
    /// Total entries in report
    pub total_entries: u32,
    /// Entries by level
    pub entries_by_level: HashMap<String, u32>,
    /// Recent diagnostic entries
    pub recent_entries: Vec<DiagnosticEntry>,
}

impl DiagnosticsReport {
    /// Create a new empty diagnostics report
    pub fn new() -> Self {
        Self {
            timestamp: chrono::Utc::now(),
            total_entries: 0,
            entries_by_level: HashMap::new(),
            recent_entries: Vec::new(),
        }
    }

    /// Add a diagnostic entry and update statistics
    pub fn add_entry(&mut self, entry: DiagnosticEntry) {
        let level_str = format!("{:?}", entry.level);
        *self.entries_by_level.entry(level_str).or_insert(0) += 1;
        self.total_entries += 1;
        self.recent_entries.push(entry);

        // Keep only last 100 entries
        if self.recent_entries.len() > 100 {
            self.recent_entries.remove(0);
        }
    }
}

impl Default for DiagnosticsReport {
    fn default() -> Self {
        Self::new()
    }
}

/// Complete telemetry packet combining health, sensor readings, and diagnostics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryPacket {
    /// Packet sequence number
    pub sequence: u64,
    /// Timestamp of packet generation
    pub timestamp: Timestamp,
    /// System health snapshot
    pub health: SystemHealth,
    /// Recent sensor readings
    pub sensor_readings: Vec<SensorReading>,
    /// Diagnostics snapshot
    pub diagnostics: DiagnosticsReport,
}

impl TelemetryPacket {
    /// Create a new telemetry packet
    pub fn new(sequence: u64) -> Self {
        Self {
            sequence,
            timestamp: chrono::Utc::now(),
            health: SystemHealth::new(),
            sensor_readings: Vec::new(),
            diagnostics: DiagnosticsReport::new(),
        }
    }

    /// Serialize to JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Serialize to JSON bytes
    pub fn to_json_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }

    /// Deserialize from JSON string
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Deserialize from JSON bytes
    pub fn from_json_bytes(bytes: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(bytes)
    }

    /// Get total size in bytes (approximate)
    pub fn size_bytes(&self) -> usize {
        self.to_json_bytes().unwrap_or_default().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_health_serialization() {
        let mut health = SystemHealth::new();
        health.healthy_components = 5;
        health.degraded_components = 1;
        health.temperature_celsius = 45.2;
        health.recalculate_status();

        let json = serde_json::to_string(&health).unwrap();
        // Status is serialized as uppercase due to serde(rename_all = "UPPERCASE")
        assert!(json.contains("DEGRADED"));

        let deserialized: SystemHealth = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.healthy_components, 5);
        assert_eq!(deserialized.status, HealthStatus::Degraded);
    }

    #[test]
    fn test_sensor_data_variants() {
        let temp = SensorData::Temperature {
            value: 25.5,
            unit: "°C".to_string(),
        };
        assert!(temp.description().contains("Temperature"));

        let gps = SensorData::Gps {
            latitude: 37.7749,
            longitude: -122.4194,
            altitude: 100.0,
            accuracy: 5.0,
        };
        assert!(gps.description().contains("GPS"));
    }

    #[test]
    fn test_sensor_reading_json() {
        let reading = SensorReading::new(
            "sensor-001".to_string(),
            "Temperature Sensor".to_string(),
            SensorData::Temperature {
                value: 22.5,
                unit: "°C".to_string(),
            },
            1,
        );

        let json = serde_json::to_string(&reading).unwrap();
        let deserialized: SensorReading = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.component_id, "sensor-001");
    }

    #[test]
    fn test_diagnostic_entry_builder() {
        let entry = DiagnosticEntry::new(
            DiagnosticLevel::Warning,
            "motor-001".to_string(),
            "Motor temperature rising",
        )
        .with_code("TEMP_WARN_001")
        .with_context("temperature".to_string(), "65°C".to_string());

        assert_eq!(entry.level, DiagnosticLevel::Warning);
        assert_eq!(entry.code, Some("TEMP_WARN_001".to_string()));
        assert!(entry.context.is_some());
    }

    #[test]
    fn test_diagnostics_report() {
        let mut report = DiagnosticsReport::new();
        report.add_entry(DiagnosticEntry::new(
            DiagnosticLevel::Info,
            "sys".to_string(),
            "System started",
        ));
        report.add_entry(DiagnosticEntry::new(
            DiagnosticLevel::Warning,
            "sensor".to_string(),
            "Sensor reading delayed",
        ));

        assert_eq!(report.total_entries, 2);
        assert_eq!(report.recent_entries.len(), 2);
    }

    #[test]
    fn test_telemetry_packet_roundtrip() {
        let mut packet = TelemetryPacket::new(1);
        packet.health.healthy_components = 10;
        packet.sensor_readings.push(SensorReading::new(
            "gps-001".to_string(),
            "GPS".to_string(),
            SensorData::Gps {
                latitude: 37.7749,
                longitude: -122.4194,
                altitude: 50.0,
                accuracy: 2.5,
            },
            1,
        ));

        let json = packet.to_json().unwrap();
        let restored = TelemetryPacket::from_json(&json).unwrap();

        assert_eq!(restored.sequence, 1);
        assert_eq!(restored.health.healthy_components, 10);
        assert_eq!(restored.sensor_readings.len(), 1);
    }

    #[test]
    fn test_telemetry_packet_size() {
        let packet = TelemetryPacket::new(1);
        let size = packet.size_bytes();
        assert!(size > 0);
    }
}
