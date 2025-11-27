//! Telemetry collector for gathering and managing system telemetry

use crate::types::*;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Collects telemetry from all system components
pub struct TelemetryCollector {
    /// Sequence number for packets
    sequence: Arc<Mutex<u64>>,
    /// System health tracking
    health: Arc<Mutex<SystemHealth>>,
    /// Diagnostics report
    diagnostics: Arc<Mutex<DiagnosticsReport>>,
    /// Recent sensor readings
    sensor_readings: Arc<Mutex<Vec<SensorReading>>>,
}

impl TelemetryCollector {
    /// Create a new telemetry collector
    pub fn new() -> Self {
        Self {
            sequence: Arc::new(Mutex::new(0)),
            health: Arc::new(Mutex::new(SystemHealth::new())),
            diagnostics: Arc::new(Mutex::new(DiagnosticsReport::new())),
            sensor_readings: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Record a sensor reading
    pub async fn record_sensor_reading(&self, reading: SensorReading) {
        let mut readings = self.sensor_readings.lock().await;
        readings.push(reading);

        // Keep only last 1000 readings
        if readings.len() > 1000 {
            readings.remove(0);
        }
    }

    /// Record a diagnostic event
    pub async fn record_diagnostic(&self, entry: DiagnosticEntry) {
        let mut diagnostics = self.diagnostics.lock().await;
        diagnostics.add_entry(entry);
    }

    /// Update system health
    pub async fn update_health(&self, health: SystemHealth) {
        let mut h = self.health.lock().await;
        *h = health;
    }

    /// Generate a complete telemetry packet
    pub async fn generate_packet(&self) -> TelemetryPacket {
        let mut seq = self.sequence.lock().await;
        *seq += 1;
        let sequence = *seq;
        drop(seq);

        let health = self.health.lock().await.clone();
        let sensor_readings = self.sensor_readings.lock().await.clone();
        let diagnostics = self.diagnostics.lock().await.clone();

        TelemetryPacket {
            sequence,
            timestamp: chrono::Utc::now(),
            health,
            sensor_readings,
            diagnostics,
        }
    }

    /// Get current health status
    pub async fn get_health(&self) -> SystemHealth {
        self.health.lock().await.clone()
    }

    /// Get recent sensor readings
    pub async fn get_sensor_readings(&self, limit: usize) -> Vec<SensorReading> {
        let readings = self.sensor_readings.lock().await;
        readings
            .iter()
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }

    /// Clear all telemetry data
    pub async fn clear(&self) {
        let mut readings = self.sensor_readings.lock().await;
        readings.clear();
    }
}

impl Default for TelemetryCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // Explicitly import std prelude to avoid collision with local `core` crate
    use std::prelude::v1::*;

    #[tokio::test]
    async fn test_collector_record_reading() {
        let collector = TelemetryCollector::new();
        let reading = SensorReading::new(
            "test-01".to_string(),
            "Test Sensor".to_string(),
            SensorData::Temperature {
                value: 25.0,
                unit: "°C".to_string(),
            },
            1,
        );

        collector.record_sensor_reading(reading.clone()).await;
        let readings = collector.get_sensor_readings(10).await;
        assert_eq!(readings.len(), 1);
        assert_eq!(readings[0].component_id, "test-01");
    }

    #[tokio::test]
    async fn test_collector_diagnostic() {
        let collector = TelemetryCollector::new();
        let entry = DiagnosticEntry::new(
            DiagnosticLevel::Info,
            "sys".to_string(),
            "Test event",
        );

        collector.record_diagnostic(entry).await;
        let packet = collector.generate_packet().await;
        assert_eq!(packet.diagnostics.total_entries, 1);
    }

    #[tokio::test]
    async fn test_collector_packet_generation() {
        let collector = TelemetryCollector::new();

        let mut health = SystemHealth::new();
        health.healthy_components = 3;
        collector.update_health(health).await;

        let packet = collector.generate_packet().await;
        assert_eq!(packet.sequence, 1);
        assert_eq!(packet.health.healthy_components, 3);
    }

    #[tokio::test]
    async fn test_collector_sequence_increment() {
        let collector = TelemetryCollector::new();
        let p1 = collector.generate_packet().await;
        let p2 = collector.generate_packet().await;
        assert_eq!(p1.sequence, 1);
        assert_eq!(p2.sequence, 2);
    }
}
