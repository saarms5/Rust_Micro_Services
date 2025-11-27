//! Async data streaming pipeline for batching, optional compression, and transport sending.
//!
//! The `StreamingPipeline` manages a bounded channel (shared bus) that decouples telemetry
//! collection from transport operations. A background task consumes packets from the bus,
//! batches them, optionally compresses, and sends via configurable transports—all without
//! blocking the main control loop.

use crate::TelemetryPacket;
use crate::transports::{Transport, TransportError, MqttTransport, SerialTransport};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::sync::mpsc::{self, Sender, Receiver};
use tokio::time::sleep;

/// Error type for streaming pipeline operations
#[derive(Error, Debug)]
pub enum StreamingError {
    #[error("Transport error: {0}")]
    Transport(#[from] TransportError),
    #[error("Channel closed")]
    ChannelClosed,
    #[error("Compression failed: {0}")]
    CompressionFailed(String),
}

/// Streaming pipeline configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineConfig {
    /// Max packets per batch before forced send
    pub batch_size: usize,
    /// Max time to wait before sending a batch (seconds)
    pub batch_timeout_secs: u64,
    /// Enable gzip compression on batches
    pub enable_compression: bool,
    /// Channel capacity (bounded buffer)
    pub channel_capacity: usize,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            batch_size: 10,
            batch_timeout_secs: 5,
            enable_compression: true,
            channel_capacity: 256,
        }
    }
}

/// Compressed batch metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressedBatch {
    /// Number of packets in this batch
    pub packet_count: usize,
    /// Compressed size in bytes
    pub compressed_size: usize,
    /// Uncompressed size estimate (sum of JSON lengths)
    pub uncompressed_size: usize,
    /// Timestamp when batch was created
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl CompressedBatch {
    /// Calculate compression ratio
    pub fn compression_ratio(&self) -> f32 {
        if self.uncompressed_size == 0 {
            1.0
        } else {
            self.compressed_size as f32 / self.uncompressed_size as f32
        }
    }
}

/// Concrete transport type for use in pipelines (avoids dyn trait issues with async methods)
pub enum PipelineTransport {
    /// MQTT adapter
    Mqtt(MqttTransport),
    /// Serial/UART adapter
    Serial(SerialTransport),
}

impl PipelineTransport {
    /// Send a packet through this transport
    pub async fn send(&self, packet: &TelemetryPacket) -> Result<(), TransportError> {
        match self {
            Self::Mqtt(t) => t.send(packet).await,
            Self::Serial(t) => t.send(packet).await,
        }
    }
}

/// Async streaming pipeline that batches packets and streams to transports.
///
/// The pipeline provides a non-blocking sender (`get_sender()`) that clients can clone
/// and use to submit packets. A background task processes the queue, batches packets,
/// optionally compresses, and forwards to one or more transports.
pub struct StreamingPipeline {
    tx: Sender<TelemetryPacket>,
    config: PipelineConfig,
    _task_handle: Arc<tokio::task::JoinHandle<()>>,
}

impl StreamingPipeline {
    /// Create a new streaming pipeline with the given config and transports.
    pub async fn new(
        config: PipelineConfig,
        transports: Vec<PipelineTransport>,
    ) -> Result<Self, StreamingError> {
        let (tx, rx) = mpsc::channel(config.channel_capacity);

        let pipeline_config = config.clone();
        let handle = tokio::spawn(Self::run_pipeline(rx, pipeline_config, transports));

        Ok(Self {
            tx,
            config,
            _task_handle: Arc::new(handle),
        })
    }

    /// Get a sender for submitting packets to the pipeline.
    /// Safe to clone and share across async tasks.
    pub fn get_sender(&self) -> Sender<TelemetryPacket> {
        self.tx.clone()
    }

    /// Main pipeline task: batch, compress, send.
    async fn run_pipeline(
        mut rx: Receiver<TelemetryPacket>,
        config: PipelineConfig,
        transports: Vec<PipelineTransport>,
    ) {
        let mut batch: Vec<TelemetryPacket> = Vec::with_capacity(config.batch_size);
        let mut batch_start = Instant::now();
        let timeout = Duration::from_secs(config.batch_timeout_secs);

        loop {
            let elapsed = batch_start.elapsed();
            let remaining = if elapsed < timeout {
                timeout - elapsed
            } else {
                Duration::from_secs(0)
            };

            tokio::select! {
                Some(packet) = rx.recv() => {
                    batch.push(packet);
                    if batch.len() >= config.batch_size {
                        if let Err(e) = Self::send_batch(&batch, &config, &transports).await {
                            eprintln!("Pipeline batch send error: {}", e);
                        }
                        batch.clear();
                        batch_start = Instant::now();
                    }
                }
                _ = sleep(remaining), if !batch.is_empty() => {
                    if let Err(e) = Self::send_batch(&batch, &config, &transports).await {
                        eprintln!("Pipeline batch send error: {}", e);
                    }
                    batch.clear();
                    batch_start = Instant::now();
                }
                else => {
                    while let Ok(packet) = rx.try_recv() {
                        batch.push(packet);
                        if batch.len() >= config.batch_size {
                            if let Err(e) = Self::send_batch(&batch, &config, &transports).await {
                                eprintln!("Pipeline batch send error: {}", e);
                            }
                            batch.clear();
                        }
                    }
                    if !batch.is_empty() {
                        if let Err(e) = Self::send_batch(&batch, &config, &transports).await {
                            eprintln!("Pipeline final batch send error: {}", e);
                        }
                    }
                    break;
                }
            }
        }
    }

    async fn send_batch(
        batch: &[TelemetryPacket],
        config: &PipelineConfig,
        transports: &[PipelineTransport],
    ) -> Result<(), StreamingError> {
        if batch.is_empty() {
            return Ok(());
        }

        let uncompressed_json = serde_json::to_string(batch)
            .map_err(|e| StreamingError::Transport(TransportError::Serialization(e)))?;
        let uncompressed_size = uncompressed_json.len();

        let _payload = if config.enable_compression {
            use flate2::Compression;
            use std::io::Write;

            let mut encoder = flate2::write::GzEncoder::new(Vec::new(), Compression::default());
            encoder
                .write_all(uncompressed_json.as_bytes())
                .map_err(|e| StreamingError::CompressionFailed(e.to_string()))?;
            encoder
                .finish()
                .map_err(|e| StreamingError::CompressionFailed(e.to_string()))?
        } else {
            uncompressed_json.into_bytes()
        };

        // Send to all transports concurrently
        let mut send_futures = Vec::new();
        for transport in transports {
            let packet = TelemetryPacket {
                sequence: batch.first().map(|p| p.sequence).unwrap_or(0),
                timestamp: chrono::Utc::now(),
                health: batch.first().map(|p| p.health.clone()).unwrap_or_default(),
                sensor_readings: batch
                    .iter()
                    .flat_map(|p| p.sensor_readings.clone())
                    .collect(),
                diagnostics: batch.first().map(|p| p.diagnostics.clone()).unwrap_or_default(),
            };
            send_futures.push(transport.send(Box::leak(Box::new(packet))));
        }
        let results = futures::future::join_all(send_futures).await;
        for result in results {
            result?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{SystemHealth, DiagnosticsReport};
    use async_trait::async_trait;
    use std::sync::{Arc, Mutex};

    struct MockTransport {
        sent_count: Arc<Mutex<usize>>,
    }

    #[async_trait]
    impl Transport for MockTransport {
        async fn send(&self, _packet: &TelemetryPacket) -> Result<(), TransportError> {
            *self.sent_count.lock().unwrap() += 1;
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_pipeline_batching() {
        let config = PipelineConfig {
            batch_size: 3,
            batch_timeout_secs: 1,
            enable_compression: false,
            channel_capacity: 256,
        };

        let mock_transport = Arc::new(MockTransport {
            sent_count: Arc::new(Mutex::new(0)),
        });

        let pipeline = StreamingPipeline::new(config, vec![mock_transport.clone()])
            .await
            .unwrap();

        let sender = pipeline.get_sender();

        // Send 3 packets (should trigger batch send)
        for i in 0..3 {
            let packet = TelemetryPacket {
                sequence: i,
                timestamp: chrono::Utc::now(),
                health: SystemHealth::new(),
                sensor_readings: vec![],
                diagnostics: DiagnosticsReport::new(),
            };
            sender.send(packet).await.unwrap();
        }

        // Give background task time to process
        tokio::time::sleep(Duration::from_millis(100)).await;

        let count = *mock_transport.sent_count.lock().unwrap();
        assert!(count > 0, "Transport should have been called");
    }

    #[tokio::test]
    async fn test_pipeline_timeout() {
        let config = PipelineConfig {
            batch_size: 100,
            batch_timeout_secs: 1,
            enable_compression: false,
            channel_capacity: 256,
        };

        let mock_transport = Arc::new(MockTransport {
            sent_count: Arc::new(Mutex::new(0)),
        });

        let pipeline = StreamingPipeline::new(config, vec![mock_transport.clone()])
            .await
            .unwrap();

        let sender = pipeline.get_sender();

        // Send 1 packet and wait for timeout
        let packet = TelemetryPacket {
            sequence: 1,
            timestamp: chrono::Utc::now(),
            health: SystemHealth::new(),
            sensor_readings: vec![],
            diagnostics: DiagnosticsReport::new(),
        };
        sender.send(packet).await.unwrap();

        // Wait for timeout (batch_timeout_secs)
        tokio::time::sleep(Duration::from_secs(2)).await;

        let count = *mock_transport.sent_count.lock().unwrap();
        assert!(count > 0, "Transport should have been called after timeout");
    }

    #[tokio::test]
    async fn test_compression_ratio() {
        let batch = CompressedBatch {
            packet_count: 10,
            compressed_size: 500,
            uncompressed_size: 1000,
            created_at: chrono::Utc::now(),
        };

        let ratio = batch.compression_ratio();
        assert!((ratio - 0.5).abs() < 0.01);
    }
}
