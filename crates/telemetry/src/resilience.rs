//! Resilience layer for telemetry pipeline
//!
//! Provides retry logic, offline buffering, and circuit breaker pattern
//! to ensure reliable delivery even under adverse conditions.

use crate::TelemetryPacket;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

/// Resilience error types
#[derive(Error, Debug)]
pub enum ResilienceError {
    #[error("Circuit breaker open: {0}")]
    CircuitBreakerOpen(String),
    #[error("Buffer full, dropped packet")]
    BufferFull,
    #[error("Retry exhausted: {0}")]
    RetryExhausted(String),
}

/// Circuit breaker state
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CircuitState {
    /// Circuit is closed, requests pass through
    Closed,
    /// Circuit is open, requests are rejected
    Open,
    /// Circuit is half-open, testing recovery
    HalfOpen,
}

/// Configuration for resilience layer
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResilienceConfig {
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Initial backoff duration
    pub initial_backoff_ms: u64,
    /// Maximum backoff duration
    pub max_backoff_ms: u64,
    /// Multiplier for exponential backoff
    pub backoff_multiplier: f64,
    /// Circuit breaker failure threshold (before opening)
    pub failure_threshold: u32,
    /// Circuit breaker half-open timeout
    pub half_open_timeout_secs: u64,
    /// Offline buffer size (max packets to buffer)
    pub buffer_size: usize,
}

impl Default for ResilienceConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_backoff_ms: 100,
            max_backoff_ms: 10000,
            backoff_multiplier: 2.0,
            failure_threshold: 5,
            half_open_timeout_secs: 30,
            buffer_size: 1000,
        }
    }
}

/// Circuit breaker for protecting against cascading failures
pub struct CircuitBreaker {
    state: Arc<RwLock<CircuitState>>,
    failure_count: Arc<AtomicU32>,
    success_count: Arc<AtomicU32>,
    failure_threshold: u32,
    half_open_timeout: Duration,
    last_open_time: Arc<RwLock<Option<std::time::Instant>>>,
}

impl CircuitBreaker {
    /// Create a new circuit breaker
    pub fn new(failure_threshold: u32, half_open_timeout_secs: u64) -> Self {
        Self {
            state: Arc::new(RwLock::new(CircuitState::Closed)),
            failure_count: Arc::new(AtomicU32::new(0)),
            success_count: Arc::new(AtomicU32::new(0)),
            failure_threshold,
            half_open_timeout: Duration::from_secs(half_open_timeout_secs),
            last_open_time: Arc::new(RwLock::new(None)),
        }
    }

    /// Get current circuit state
    pub async fn state(&self) -> CircuitState {
        *self.state.read().await
    }

    /// Record a successful operation
    pub async fn record_success(&self) {
        let state = self.state.read().await;
        match *state {
            CircuitState::Closed => {
                // Reset failures on success in closed state
                self.failure_count.store(0, Ordering::SeqCst);
            }
            CircuitState::HalfOpen => {
                // Transition to closed after 3 successes
                let success = self.success_count.fetch_add(1, Ordering::SeqCst) + 1;
                if success >= 3 {
                    drop(state);
                    *self.state.write().await = CircuitState::Closed;
                    self.failure_count.store(0, Ordering::SeqCst);
                    self.success_count.store(0, Ordering::SeqCst);
                }
            }
            _ => {}
        }
    }

    /// Record a failed operation
    pub async fn record_failure(&self) {
        let mut state = self.state.write().await;
        match *state {
            CircuitState::Closed => {
                let failures = self.failure_count.fetch_add(1, Ordering::SeqCst) + 1;
                if failures >= self.failure_threshold {
                    *state = CircuitState::Open;
                    *self.last_open_time.write().await = Some(std::time::Instant::now());
                    tracing::warn!("Circuit breaker opened after {} failures", failures);
                }
            }
            CircuitState::HalfOpen => {
                // Any failure in half-open state returns to open
                *state = CircuitState::Open;
                *self.last_open_time.write().await = Some(std::time::Instant::now());
                self.success_count.store(0, Ordering::SeqCst);
            }
            _ => {}
        }
    }

    /// Try to transition from open to half-open if timeout elapsed
    pub async fn try_half_open(&self) {
        let state = self.state.read().await;
        if *state != CircuitState::Open {
            return;
        }
        drop(state);

        if let Some(last_open) = *self.last_open_time.read().await {
            if last_open.elapsed() >= self.half_open_timeout {
                *self.state.write().await = CircuitState::HalfOpen;
                self.failure_count.store(0, Ordering::SeqCst);
                self.success_count.store(0, Ordering::SeqCst);
                tracing::info!("Circuit breaker transitioning to half-open");
            }
        }
    }
}

/// Offline buffer for storing packets when transport is unavailable
pub struct OfflineBuffer {
    packets: Arc<RwLock<Vec<TelemetryPacket>>>,
    max_size: usize,
}

impl OfflineBuffer {
    /// Create a new offline buffer
    pub fn new(max_size: usize) -> Self {
        Self {
            packets: Arc::new(RwLock::new(Vec::with_capacity(max_size))),
            max_size,
        }
    }

    /// Add a packet to the buffer
    pub async fn push(&self, packet: TelemetryPacket) -> Result<(), ResilienceError> {
        let mut packets = self.packets.write().await;
        if packets.len() >= self.max_size {
            return Err(ResilienceError::BufferFull);
        }
        packets.push(packet);
        Ok(())
    }

    /// Get and remove the next packet from the buffer
    pub async fn pop(&self) -> Option<TelemetryPacket> {
        let mut packets = self.packets.write().await;
        if packets.is_empty() {
            None
        } else {
            Some(packets.remove(0))
        }
    }

    /// Get current buffer size
    pub async fn len(&self) -> usize {
        self.packets.read().await.len()
    }

    /// Get all packets and clear buffer
    pub async fn drain(&self) -> Vec<TelemetryPacket> {
        let mut packets = self.packets.write().await;
        packets.drain(..).collect()
    }
}

/// Retry strategy with exponential backoff
pub struct RetryStrategy {
    config: ResilienceConfig,
}

impl RetryStrategy {
    /// Create a new retry strategy
    pub fn new(config: ResilienceConfig) -> Self {
        Self { config }
    }

    /// Execute operation with retry logic (simplified without backoff closure capture issue)
    pub async fn execute_simple<F, T>(&self, mut f: F) -> Result<T, ResilienceError>
    where
        F: FnMut() -> T,
    {
        let mut attempt = 0;
        let max_attempts = self.config.max_retries as usize;
        let mut current_backoff = self.config.initial_backoff_ms;

        loop {
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| f())) {
                Ok(result) => return Ok(result),
                Err(_) => {
                    attempt += 1;
                    if attempt >= max_attempts {
                        return Err(ResilienceError::RetryExhausted(
                            "Max retries exceeded".to_string(),
                        ));
                    }
                    tokio::time::sleep(Duration::from_millis(current_backoff)).await;
                    current_backoff = std::cmp::min(
                        (current_backoff as f64 * self.config.backoff_multiplier) as u64,
                        self.config.max_backoff_ms,
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_circuit_breaker_transitions() {
        let breaker = CircuitBreaker::new(3, 1);
        assert_eq!(breaker.state().await, CircuitState::Closed);

        // Record 3 failures
        for _ in 0..3 {
            breaker.record_failure().await;
        }
        assert_eq!(breaker.state().await, CircuitState::Open);

        // Transition to half-open after timeout
        tokio::time::sleep(Duration::from_secs(2)).await;
        breaker.try_half_open().await;
        assert_eq!(breaker.state().await, CircuitState::HalfOpen);

        // Record 3 successes to close
        for _ in 0..3 {
            breaker.record_success().await;
        }
        assert_eq!(breaker.state().await, CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_offline_buffer() {
        let buffer = OfflineBuffer::new(3);
        let packet = TelemetryPacket::new(
            crate::types::ComponentId::from("test"),
            crate::types::Timestamp::now(),
        );

        assert!(buffer.push(packet.clone()).await.is_ok());
        assert_eq!(buffer.len().await, 1);

        let popped = buffer.pop().await;
        assert!(popped.is_some());
        assert_eq!(buffer.len().await, 0);
    }

    #[test]
    fn test_default_resilience_config() {
        let config = ResilienceConfig::default();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.failure_threshold, 5);
        assert_eq!(config.buffer_size, 1000);
    }
}
