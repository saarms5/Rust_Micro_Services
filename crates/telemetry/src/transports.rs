use crate::TelemetryPacket;
use async_trait::async_trait;
use serde_json;
use std::path::PathBuf;
use std::sync::Arc;
use thiserror::Error;
use tokio::fs::{self, OpenOptions};
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc::{self, Sender};

/// Error type for transport operations
#[derive(Error, Debug)]
pub enum TransportError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Transport closed or channel error")]
    Closed,
    #[error("Other: {0}")]
    Other(String),
}

#[async_trait]
pub trait Transport: Send + Sync {
    /// Send a telemetry packet over this transport
    async fn send(&self, packet: &TelemetryPacket) -> Result<(), TransportError>;
}

/// Simple MQTT transport adapter.
///
/// By default this adapter serializes `TelemetryPacket` to JSON and appends to a file
/// `telemetry_out/mqtt_publish.log`. This provides an out-of-the-box, deterministic
/// implementation for testing and local monitoring. Replace with a real MQTT client
/// (e.g. `rumqttc`) behind a feature flag when needed.
pub struct MqttTransport {
    tx: Sender<String>,
    _task_handle: Arc<tokio::task::JoinHandle<()>>,
}

impl MqttTransport {
    /// Create a new MQTT transport that writes JSON messages to `out_path`.
    pub async fn new(out_path: Option<PathBuf>) -> Result<Self, TransportError> {
        let path = out_path.unwrap_or_else(|| PathBuf::from("telemetry_out/mqtt_publish.log"));
        let parent_dir = path
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."));
        fs::create_dir_all(&parent_dir).await?;

        let (tx, mut rx) = mpsc::channel::<String>(256);

        // spawn background task to consume channel and write to file
        let file_path = path.clone();
        let handle = tokio::spawn(async move {
            // open in append mode
            let mut file = match OpenOptions::new()
                .create(true)
                .append(true)
                .open(&file_path)
                .await
            {
                Ok(f) => f,
                Err(e) => {
                    eprintln!(
                        "MqttTransport failed to open file {}: {}",
                        file_path.display(),
                        e
                    );
                    return;
                }
            };

            while let Some(msg) = rx.recv().await {
                if let Err(e) = file.write_all(msg.as_bytes()).await {
                    eprintln!("MqttTransport write error: {}", e);
                    // best-effort continue
                }
                if let Err(e) = file.write_all(b"\n").await {
                    eprintln!("MqttTransport write newline error: {}", e);
                }
                // flush occasionally
                let _ = file.flush().await;
            }
        });

        Ok(Self {
            tx,
            _task_handle: Arc::new(handle),
        })
    }
}

#[async_trait]
impl Transport for MqttTransport {
    async fn send(&self, packet: &TelemetryPacket) -> Result<(), TransportError> {
        let json = serde_json::to_string(packet)?;
        self.tx.send(json).await.map_err(|_| TransportError::Closed)
    }
}

/// Simple Serial/UART transport adapter.
///
/// By default this adapter serializes `TelemetryPacket` to JSON and appends to a file
/// `telemetry_out/serial.log`. Replace with `tokio-serial` or another serial library
/// behind a feature flag for real hardware.
pub struct SerialTransport {
    tx: Sender<String>,
    _task_handle: Arc<tokio::task::JoinHandle<()>>,
}

impl SerialTransport {
    /// Create a new Serial transport that writes JSON messages to `out_path`.
    pub async fn new(out_path: Option<PathBuf>) -> Result<Self, TransportError> {
        let path = out_path.unwrap_or_else(|| PathBuf::from("telemetry_out/serial.log"));
        let parent_dir = path
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."));
        fs::create_dir_all(&parent_dir).await?;

        let (tx, mut rx) = mpsc::channel::<String>(256);
        let file_path = path.clone();

        let handle = tokio::spawn(async move {
            let mut file = match OpenOptions::new()
                .create(true)
                .append(true)
                .open(&file_path)
                .await
            {
                Ok(f) => f,
                Err(e) => {
                    eprintln!(
                        "SerialTransport failed to open file {}: {}",
                        file_path.display(),
                        e
                    );
                    return;
                }
            };

            while let Some(msg) = rx.recv().await {
                if let Err(e) = file.write_all(msg.as_bytes()).await {
                    eprintln!("SerialTransport write error: {}", e);
                }
                if let Err(e) = file.write_all(b"\n").await {
                    eprintln!("SerialTransport write newline error: {}", e);
                }
                let _ = file.flush().await;
            }
        });

        Ok(Self {
            tx,
            _task_handle: Arc::new(handle),
        })
    }
}

#[async_trait]
impl Transport for SerialTransport {
    async fn send(&self, packet: &TelemetryPacket) -> Result<(), TransportError> {
        let json = serde_json::to_string(packet)?;
        self.tx.send(json).await.map_err(|_| TransportError::Closed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{SystemHealth, TelemetryPacket};

    #[tokio::test]
    async fn test_mqtt_transport_send() {
        let transport = MqttTransport::new(Some(PathBuf::from("target/test_output/mqtt_test.log")))
            .await
            .unwrap();
        let packet = TelemetryPacket {
            sequence: 1,
            timestamp: chrono::Utc::now(),
            health: SystemHealth::new(),
            sensor_readings: vec![],
            diagnostics: Default::default(),
        };

        transport.send(&packet).await.unwrap();
    }

    #[tokio::test]
    async fn test_serial_transport_send() {
        let transport =
            SerialTransport::new(Some(PathBuf::from("target/test_output/serial_test.log")))
                .await
                .unwrap();
        let packet = TelemetryPacket {
            sequence: 2,
            timestamp: chrono::Utc::now(),
            health: SystemHealth::new(),
            sensor_readings: vec![],
            diagnostics: Default::default(),
        };

        transport.send(&packet).await.unwrap();
    }
}
