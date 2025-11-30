#![cfg(feature = "mqtt_real")]

//! Production-grade MQTT transport using `rumqttc`.
//!
//! This module provides a robust MQTT client with:
//! - Connection retry logic with exponential backoff
//! - TLS/mTLS support
//! - Configurable QoS
//! - Automatic reconnection

use crate::TelemetryPacket;
use async_trait::async_trait;
use backoff::future::retry;
use backoff::ExponentialBackoff;
use rumqttc::{AsyncClient, MqttOptions, QoS, TlsConfiguration};
use serde_json;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::sync::Mutex;

use super::Transport;

/// Error type for MQTT operations
#[derive(Error, Debug)]
pub enum MqttError {
    #[error("MQTT connection error: {0}")]
    Connection(String),
    #[error("MQTT publish error: {0}")]
    Publish(String),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Not connected")]
    NotConnected,
    #[error("Client not initialized")]
    ClientNotInitialized,
    #[error("TLS configuration error: {0}")]
    TlsConfig(String),
}

/// Configuration for real MQTT transport
#[derive(Clone, Debug)]
pub struct MqttConfig {
    /// MQTT broker host (e.g., "mqtt.example.com")
    pub host: String,
    /// MQTT broker port (default 1883, 8883 for TLS)
    pub port: u16,
    /// Client ID for MQTT connection
    pub client_id: String,
    /// Topic to publish telemetry to
    pub topic: String,
    /// QoS level (0, 1, or 2)
    pub qos: u8,
    /// Keep alive interval in seconds
    pub keep_alive_secs: u64,
    /// Use TLS
    pub use_tls: bool,
    /// CA certificate path (for TLS)
    pub ca_cert_path: Option<String>,
    /// Client certificate path (for mTLS)
    pub client_cert_path: Option<String>,
    /// Client key path (for mTLS)
    pub client_key_path: Option<String>,
    /// Maximum reconnection attempts (0 = infinite)
    pub max_reconnect_attempts: u32,
}

impl Default for MqttConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 1883,
            client_id: format!("rust-telemetry-{}", std::process::id()),
            topic: "telemetry/system".to_string(),
            qos: 1,
            keep_alive_secs: 60,
            use_tls: false,
            ca_cert_path: None,
            client_cert_path: None,
            client_key_path: None,
            max_reconnect_attempts: 0, // infinite retries
        }
    }
}

/// Production MQTT transport with reconnection and retry logic
pub struct RealMqttTransport {
    config: MqttConfig,
    client: Arc<Mutex<Option<AsyncClient>>>,
    connected: Arc<AtomicBool>,
    rx_handle: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
}

impl RealMqttTransport {
    /// Create a new real MQTT transport and connect
    pub async fn new(config: MqttConfig) -> Result<Self, MqttError> {
        let transport = Self {
            config: config.clone(),
            client: Arc::new(Mutex::new(None)),
            connected: Arc::new(AtomicBool::new(false)),
            rx_handle: Arc::new(Mutex::new(None)),
        };

        transport.connect().await?;
        Ok(transport)
    }

    /// Connect to MQTT broker with retry logic
    async fn connect(&self) -> Result<(), MqttError> {
        let backoff = ExponentialBackoff {
            max_elapsed_time: None,
            ..Default::default()
        };

        let config = self.config.clone();
        let client_arc = self.client.clone();
        let connected_arc = self.connected.clone();
        let rx_handle_arc = self.rx_handle.clone();

        retry(backoff, || async {
            let mut mqtt_opts =
                MqttOptions::new(config.client_id.clone(), config.host.clone(), config.port);
            mqtt_opts.set_keep_alive(Duration::from_secs(config.keep_alive_secs));

            // Configure TLS if needed
            if config.use_tls {
                match Self::configure_tls(&config).await {
                    Ok(tls_config) => {
                        mqtt_opts.set_transport(rumqttc::Transport::Tls(tls_config));
                    }
                    Err(e) => return Err(e),
                }
            }

            let (client, mut eventloop) = AsyncClient::new(mqtt_opts, 100);

            // Spawn event loop handler
            let connected = connected_arc.clone();
            let client_handle = tokio::spawn(async move {
                loop {
                    match eventloop.poll().await {
                        Ok(notification) => {
                            use rumqttc::Event;
                            match notification {
                                Event::Incoming(rumqttc::Incoming::ConnAck(_)) => {
                                    connected.store(true, Ordering::SeqCst);
                                }
                                Event::Incoming(rumqttc::Incoming::Disconnect) => {
                                    connected.store(false, Ordering::SeqCst);
                                }
                                _ => {}
                            }
                        }
                        Err(e) => {
                            tracing::warn!("MQTT event loop error: {:?}", e);
                            connected.store(false, Ordering::SeqCst);
                            break;
                        }
                    }
                }
            });

            *client_arc.lock().await = Some(client);
            *rx_handle_arc.lock().await = Some(client_handle);
            connected_arc.store(true, Ordering::SeqCst);

            Ok(())
        })
        .await
        .map_err(|e| MqttError::Connection(format!("Failed to connect after retries: {}", e)))?;

        Ok(())
    }

    /// Configure TLS/mTLS
    async fn configure_tls(config: &MqttConfig) -> Result<TlsConfiguration, MqttError> {
        use std::fs;

        // Load CA certificate
        let ca_cert = if let Some(ca_path) = &config.ca_cert_path {
            fs::read(ca_path)
                .map_err(|e| MqttError::TlsConfig(format!("Failed to read CA cert: {}", e)))?
        } else {
            // Use native system CA if not specified
            return Ok(TlsConfiguration::default());
        };

        // Load client certificate and key if provided (mTLS)
        let (client_cert, client_key) = if let (Some(cert_path), Some(key_path)) =
            (&config.client_cert_path, &config.client_key_path)
        {
            let cert = fs::read(cert_path)
                .map_err(|e| MqttError::TlsConfig(format!("Failed to read client cert: {}", e)))?;
            let key = fs::read(key_path)
                .map_err(|e| MqttError::TlsConfig(format!("Failed to read client key: {}", e)))?;
            (Some(cert), Some(key))
        } else {
            (None, None)
        };

        Ok(TlsConfiguration::Simple {
            ca: ca_cert,
            alpn: None,
            client_auth: if client_cert.is_some() && client_key.is_some() {
                Some((client_cert.unwrap(), client_key.unwrap()))
            } else {
                None
            },
        })
    }

    /// Check if connected and reconnect if needed
    async fn ensure_connected(&self) -> Result<(), MqttError> {
        if !self.connected.load(Ordering::SeqCst) {
            self.connect().await?;
        }
        Ok(())
    }
}

#[async_trait]
impl Transport for RealMqttTransport {
    async fn send(&self, packet: &TelemetryPacket) -> Result<(), super::TransportError> {
        self.ensure_connected()
            .await
            .map_err(|e| super::TransportError::Other(e.to_string()))?;

        let json = serde_json::to_vec(packet)?;

        let client = self.client.lock().await;
        if let Some(ref c) = *client {
            let qos = match self.config.qos {
                0 => QoS::AtMostOnce,
                1 => QoS::AtLeastOnce,
                2 => QoS::ExactlyOnce,
                _ => QoS::AtLeastOnce,
            };

            c.publish(self.config.topic.clone(), qos, false, json)
                .await
                .map_err(|e| super::TransportError::Other(format!("MQTT publish failed: {}", e)))?;
            Ok(())
        } else {
            Err(super::TransportError::Other(
                "MQTT client not initialized".to_string(),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = MqttConfig::default();
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 1883);
        assert_eq!(config.qos, 1);
    }
}
