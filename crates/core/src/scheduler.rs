//! Real-time scheduler for deterministic, fixed-frequency control loops
//!
//! This module implements a scheduler that can run high-priority control loops
//! at guaranteed frequencies (e.g., 100Hz) while handling lower-priority async
//! tasks concurrently.

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

/// Result type for scheduler operations
pub type SchedulerResult<T> = Result<T, SchedulerError>;

/// Error type for scheduler operations
#[derive(Debug, Clone)]
pub enum SchedulerError {
    LoopMissedDeadline,
    TaskExecutionError(String),
    InvalidFrequency,
}

impl std::fmt::Display for SchedulerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LoopMissedDeadline => write!(f, "Control loop missed deadline"),
            Self::TaskExecutionError(msg) => write!(f, "Task execution error: {}", msg),
            Self::InvalidFrequency => write!(f, "Invalid frequency specified"),
        }
    }
}

impl std::error::Error for SchedulerError {}

/// Statistics for a control loop execution
#[derive(Debug, Clone, Copy)]
pub struct LoopStats {
    /// Desired period between iterations (e.g., 10ms for 100Hz)
    pub period_ms: u32,
    /// Actual measured period in milliseconds
    pub measured_period_ms: u32,
    /// Time spent in user code (milliseconds)
    pub execution_time_ms: u32,
    /// Slack time before next deadline (milliseconds)
    pub slack_time_ms: i32,
    /// Number of iterations completed
    pub iteration_count: u64,
}

impl LoopStats {
    /// Calculate utilization as a percentage (0-100)
    pub fn utilization_percent(&self) -> f32 {
        (self.execution_time_ms as f32 / self.period_ms as f32) * 100.0
    }
}

/// Trait for tasks that must run at fixed, guaranteed frequencies
pub trait ControlLoopTask: Send {
    /// Execute one iteration of the control loop
    ///
    /// This should be quick (no blocking, no long async waits).
    /// Target execution time should be well under the loop period.
    fn execute(&mut self) -> SchedulerResult<()>;

    /// Optional: Get the name of this task for logging
    fn name(&self) -> &str {
        "ControlLoopTask"
    }
}

/// A rate-limited, real-time control loop scheduler
///
/// Guarantees that a task will execute at a specified frequency,
/// with monitoring for deadline misses.
pub struct RealTimeLoop {
    /// Desired execution frequency in Hz
    frequency_hz: u32,
    /// Period between iterations
    period: Duration,
    /// Statistics tracking
    stats: LoopStats,
    /// Time of last iteration start
    last_iteration: Instant,
    /// Last measured period
    measured_period: Duration,
}

impl RealTimeLoop {
    /// Create a new real-time loop at a specified frequency
    ///
    /// # Arguments
    /// * `frequency_hz` - Desired frequency in Hz (e.g., 100 for 100Hz)
    ///
    /// # Returns
    /// SchedulerResult containing the loop or an error if frequency is invalid
    pub fn new(frequency_hz: u32) -> SchedulerResult<Self> {
        if frequency_hz == 0 || frequency_hz > 10000 {
            return Err(SchedulerError::InvalidFrequency);
        }

        let period_ms = 1000 / frequency_hz;
        let period = Duration::from_millis(period_ms as u64);

        Ok(Self {
            frequency_hz,
            period,
            stats: LoopStats {
                period_ms,
                measured_period_ms: 0,
                execution_time_ms: 0,
                slack_time_ms: 0,
                iteration_count: 0,
            },
            last_iteration: Instant::now(),
            measured_period: Duration::ZERO,
        })
    }

    /// Wait until the next period boundary, maintaining guaranteed frequency
    ///
    /// This should be called at the end of each iteration.
    pub async fn wait_next_period(&mut self) {
        let elapsed = self.last_iteration.elapsed();

        // Calculate how long to sleep to maintain frequency
        if elapsed < self.period {
            let sleep_time = self.period - elapsed;
            tokio::time::sleep(sleep_time).await;
        } else if elapsed > self.period {
            // Missed deadline warning
            eprintln!(
                "[{}Hz Loop] Warning: Missed deadline by {:.1}ms",
                self.frequency_hz,
                (elapsed - self.period).as_secs_f32() * 1000.0
            );
        }

        // Update statistics
        let now = Instant::now();
        self.measured_period = now - self.last_iteration;
        self.stats.measured_period_ms = self.measured_period.as_millis() as u32;
        self.stats.execution_time_ms = elapsed.as_millis() as u32;
        self.stats.slack_time_ms = (self.period.as_millis() as i32) - (elapsed.as_millis() as i32);
        self.stats.iteration_count += 1;
        self.last_iteration = now;
    }

    /// Get current loop statistics
    pub fn stats(&self) -> LoopStats {
        self.stats
    }

    /// Log current loop statistics
    pub fn log_stats(&self) {
        println!(
            "[{}Hz Loop] Iteration {}: Exec {:.1}ms, Period {:.1}ms, Slack {:.1}ms, Util {:.1}%",
            self.frequency_hz,
            self.stats.iteration_count,
            self.stats.execution_time_ms,
            self.stats.measured_period_ms,
            self.stats.slack_time_ms,
            self.stats.utilization_percent(),
        );
    }
}

/// Mixed-priority runtime that runs high-frequency control loops
/// alongside lower-priority async tasks
pub struct MixedPriorityRuntime {
    /// Control loop frequency in Hz
    loop_frequency: u32,
    /// Background async tasks (reserved for future use)
    #[allow(dead_code)]
    background_tasks: Arc<Mutex<Vec<Box<dyn std::any::Any + Send>>>>,
    /// Cancellation token
    shutdown_token: CancellationToken,
}

impl MixedPriorityRuntime {
    /// Create a new mixed-priority runtime
    pub fn new(loop_frequency: u32) -> SchedulerResult<Self> {
        if loop_frequency == 0 || loop_frequency > 10000 {
            return Err(SchedulerError::InvalidFrequency);
        }

        Ok(Self {
            loop_frequency,
            background_tasks: Arc::new(Mutex::new(Vec::new())),
            shutdown_token: CancellationToken::new(),
        })
    }

    /// Run a control loop task at guaranteed frequency with background async support
    ///
    /// # Arguments
    /// * `task` - The control loop task to execute
    /// * `shutdown` - Cancellation token to stop execution
    ///
    /// # Example
    /// ```ignore
    /// let mut loop = RealTimeLoop::new(100)?; // 100Hz
    /// runtime.run_control_loop(&mut my_task, shutdown_token).await?;
    /// ```
    pub async fn run_control_loop(
        &self,
        task: &mut dyn ControlLoopTask,
        shutdown: CancellationToken,
    ) -> SchedulerResult<()> {
        let mut loop_scheduler = RealTimeLoop::new(self.loop_frequency)?;

        println!(
            "[{}Hz Control Loop] Starting: {}",
            self.loop_frequency,
            task.name()
        );

        loop {
            tokio::select! {
                biased;
                _ = shutdown.cancelled() => {
                    println!(
                        "[{}Hz Loop] Shutdown requested after {} iterations",
                        self.loop_frequency,
                        loop_scheduler.stats().iteration_count
                    );
                    loop_scheduler.log_stats();
                    return Ok(());
                }
                _ = tokio::time::sleep(Duration::from_millis(1)) => {
                    // Check for shutdown without blocking
                    if shutdown.is_cancelled() {
                        continue;
                    }

                    // Execute the control loop task
                    task.execute()?;

                    // Wait until next period to maintain frequency
                    loop_scheduler.wait_next_period().await;

                    // Periodically log statistics
                    if loop_scheduler.stats().iteration_count % (self.loop_frequency as u64) == 0 {
                        loop_scheduler.log_stats();
                    }
                }
            }
        }
    }

    /// Get shutdown token for coordinating multiple loops
    pub fn shutdown_token(&self) -> CancellationToken {
        self.shutdown_token.clone()
    }

    /// Request graceful shutdown
    pub fn request_shutdown(&self) {
        self.shutdown_token.cancel();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(dead_code)]
    struct TestTask {
        iterations: u32,
    }

    impl ControlLoopTask for TestTask {
        fn execute(&mut self) -> SchedulerResult<()> {
            self.iterations += 1;
            Ok(())
        }

        fn name(&self) -> &str {
            "TestTask"
        }
    }

    #[test]
    fn test_real_time_loop_creation() {
        let loop_100hz = RealTimeLoop::new(100);
        assert!(loop_100hz.is_ok());

        let loop_invalid = RealTimeLoop::new(0);
        assert!(loop_invalid.is_err());

        let loop_too_high = RealTimeLoop::new(20000);
        assert!(loop_too_high.is_err());
    }

    #[test]
    fn test_loop_stats_utilization() {
        let stats = LoopStats {
            period_ms: 10,
            measured_period_ms: 10,
            execution_time_ms: 5,
            slack_time_ms: 5,
            iteration_count: 0,
        };

        assert_eq!(stats.utilization_percent(), 50.0);
    }
}
