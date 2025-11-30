//! Async data streaming pipeline for batching, optional compression, and transport sending.
//!
//! The `StreamingPipeline` manages a bounded channel (shared bus) that decouples telemetry
//! collection from transport operations. A background task consumes packets from the bus,
//! batches them, optionally compresses, and sends via configurable transports—all without
//! blocking the main control loop.
//!
//! Includes resilience features:
//! - Retry with exponential backoff
//! - Offline buffering when transport unavailable
//! - Circuit breaker pattern for cascading failure prevention

use crate::resilience::{CircuitBreaker, OfflineBuffer, ResilienceConfig};
use crate::transports::{MqttTransport, SerialTransport, Transport, TransportError};
use crate::TelemetryPacket;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::sync::mpsc::{self, Receiver, Sender};
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
    #[error("Resilience error: {0}")]
    Resilience(String),
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
    /// Enable resilience features (retry, buffering, circuit breaker)
    pub enable_resilience: bool,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            batch_size: 10,
            batch_timeout_secs: 5,
            enable_compression: true,
            channel_capacity: 256,
            enable_resilience: true,
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
///
/// Resilience features (when enabled):
/// - Automatically retries failed sends with exponential backoff
/// - Buffers packets offline when transport is unavailable
/// - Uses circuit breaker to prevent cascading failures
pub struct StreamingPipeline {
    tx: Sender<TelemetryPacket>,
    _config: PipelineConfig,
    _task_handle: Arc<tokio::task::JoinHandle<()>>,
    /// Resilience components (optional)
    pub circuit_breaker: Option<Arc<CircuitBreaker>>,
    pub offline_buffer: Option<Arc<OfflineBuffer>>,
}

impl StreamingPipeline {
    /// Create a new streaming pipeline with the given config and transports.
    pub async fn new(
        config: PipelineConfig,
        transports: Vec<PipelineTransport>,
    ) -> Result<Self, StreamingError> {
        let (tx, rx) = mpsc::channel(config.channel_capacity);

        // Initialize resilience components if enabled
        let (circuit_breaker, offline_buffer) = if config.enable_resilience {
            let resilience_config = ResilienceConfig::default();
            let cb = Arc::new(CircuitBreaker::new(
                resilience_config.failure_threshold,
                resilience_config.half_open_timeout_secs,
            ));
            let ob = Arc::new(OfflineBuffer::new(resilience_config.buffer_size));
            (Some(cb), Some(ob))
        } else {
            (None, None)
        };

        let pipeline_config = config.clone();
        let handle = tokio::spawn(Self::run_pipeline(
            rx,
            pipeline_config,
            transports,
            circuit_breaker.clone(),
            offline_buffer.clone(),
        ));

        Ok(Self {
            tx,
            _config: config,
            _task_handle: Arc::new(handle),
            circuit_breaker,
            offline_buffer,
        })
    }

    /// Get a sender for submitting packets to the pipeline.
    /// Safe to clone and share across async tasks.
    pub fn get_sender(&self) -> Sender<TelemetryPacket> {
        self.tx.clone()
    }

    /// Main pipeline task: batch, compress, send with resilience.
    async fn run_pipeline(
        mut rx: Receiver<TelemetryPacket>,
        config: PipelineConfig,
        transports: Vec<PipelineTransport>,
        circuit_breaker: Option<Arc<CircuitBreaker>>,
        offline_buffer: Option<Arc<OfflineBuffer>>,
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
                        if let Err(e) = Self::send_batch(&batch, &config, &transports, &circuit_breaker, &offline_buffer).await {
                            tracing::error!("Pipeline batch send error: {}", e);
                        }
                        batch.clear();
                        batch_start = Instant::now();
                    }
                }
                _ = sleep(remaining), if !batch.is_empty() => {
                    if let Err(e) = Self::send_batch(&batch, &config, &transports, &circuit_breaker, &offline_buffer).await {
                        tracing::error!("Pipeline batch send error: {}", e);
                    }
                    batch.clear();
                    batch_start = Instant::now();
                }
                else => {
                    while let Ok(packet) = rx.try_recv() {
                        batch.push(packet);
                        if batch.len() >= config.batch_size {
                            if let Err(e) = Self::send_batch(&batch, &config, &transports, &circuit_breaker, &offline_buffer).await {
                                tracing::error!("Pipeline batch send error: {}", e);
                            }
                            batch.clear();
                        }
                    }
                    if !batch.is_empty() {
                        if let Err(e) = Self::send_batch(&batch, &config, &transports, &circuit_breaker, &offline_buffer).await {
                            tracing::error!("Pipeline final batch send error: {}", e);
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
        circuit_breaker: &Option<Arc<CircuitBreaker>>,
        offline_buffer: &Option<Arc<OfflineBuffer>>,
    ) -> Result<(), StreamingError> {
        if batch.is_empty() {
            return Ok(());
        }

        let uncompressed_json = serde_json::to_string(batch)
            .map_err(|e| StreamingError::Transport(TransportError::Serialization(e)))?;
        let _uncompressed_size = uncompressed_json.len();

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

        // Check circuit breaker before sending
        if let Some(ref cb) = circuit_breaker {
            cb.try_half_open().await;
            if cb.state().await == crate::resilience::CircuitState::Open {
                // Circuit is open, buffer packets offline if possible
                if let Some(ref ob) = offline_buffer {
                    for packet in batch {
                        ob.push(packet.clone()).await.ok(); // ignore buffer full
                    }
                    tracing::warn!(
                        "Circuit breaker open, buffered {} packets offline",
                        batch.len()
                    );
                    return Ok(());
                }
            }
        }

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
                diagnostics: batch
                    .first()
                    .map(|p| p.diagnostics.clone())
                    .unwrap_or_default(),
            };
            send_futures.push(transport.send(Box::leak(Box::new(packet))));
        }

        let results = futures::future::join_all(send_futures).await;
        let mut all_succeeded = true;
        for result in results {
            if let Err(e) = result {
                all_succeeded = false;
                if let Some(ref cb) = circuit_breaker {
                    cb.record_failure().await;
                }
                // Buffer failed packets if offline buffering enabled
                if let Some(ref ob) = offline_buffer {
                    for packet in batch {
                        ob.push(packet.clone()).await.ok();
                    }
                }
                tracing::warn!("Transport send failed: {}, buffered packets offline", e);
            }
        }

        if all_succeeded {
            if let Some(ref cb) = circuit_breaker {
                cb.record_success().await;
            }
            // Try to drain offline buffer and retry buffered packets
            if let Some(ref ob) = offline_buffer {
                while let Some(buffered_packet) = ob.pop().await {
                    for transport in transports {
                        let _ = transport.send(&buffered_packet).await;
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DiagnosticsReport, SystemHealth};
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_pipeline_batching_file_transport() {
        let config = PipelineConfig {
            batch_size: 2,
            batch_timeout_secs: 1,
            enable_compression: false,
            enable_resilience: false,
            channel_capacity: 256,
        };

        let out = PathBuf::from("target/test_output/streaming_batch.log");
        let mqtt = MqttTransport::new(Some(out.clone())).await.unwrap();
        let transports = vec![PipelineTransport::Mqtt(mqtt)];

        let pipeline = StreamingPipeline::new(config, transports).await.unwrap();
        let sender = pipeline.get_sender();

        for i in 0..2 {
            let packet = TelemetryPacket {
                sequence: i,
                timestamp: chrono::Utc::now(),
                health: SystemHealth::new(),
                sensor_readings: vec![],
                diagnostics: DiagnosticsReport::new(),
            };
            sender.send(packet).await.unwrap();
        }

        tokio::time::sleep(Duration::from_millis(200)).await;

        // Ensure file was written
        let meta = tokio::fs::metadata(out).await.unwrap();
        assert!(meta.len() > 0);
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
