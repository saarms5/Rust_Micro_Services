use core::{ComponentManager, MotorActuator, TemperatureSensor};
use telemetry::{Logger, LogLevel};

// Only include the runtime helper when the tokio runtime feature is enabled.
#[cfg(feature = "tokio_runtime")]
mod runtime;

#[cfg(feature = "tokio_runtime")]
async fn async_main() {
    use std::sync::Arc;
    use tokio::sync::Mutex;

    println!("=== Rust Microservices Application ===\n");

    let logger = Arc::new(Logger::new(LogLevel::Info));
    logger.log(LogLevel::Info, "Application starting...");

    // Create component manager wrapped for shared async access
    let manager = Arc::new(Mutex::new(ComponentManager::new()));

    // Register components
    let sensor = Box::new(TemperatureSensor::new("sensor-001", "Temperature Sensor"));
    let motor = Box::new(MotorActuator::new("motor-001", "Main Motor"));

    {
        let mut mgr = manager.lock().await;
        mgr.register(sensor);
        mgr.register(motor);
    }

    // Initialize all components
    println!("\n--- Initialization Phase ---");
    {
        let mut mgr = manager.lock().await;
        if let Err(e) = mgr.init_all().await {
            logger.log(LogLevel::Error, &format!("Initialization failed: {}", e));
            return;
        }
    }

    // Health check
    println!("\n--- Health Check Phase ---");
    {
        let mgr = manager.lock().await;
        if let Err(e) = mgr.health_check_all().await {
            logger.log(LogLevel::Error, &format!("Health check failed: {}", e));
        } else {
            logger.log(LogLevel::Info, "All components healthy");
        }
    }

    // Run all components with Ctrl-C graceful shutdown support
    println!("\n--- Execution Phase ---");

    use tokio_util::sync::CancellationToken;

    let manager_run = manager.clone();
    let logger_run = logger.clone();

    // Create a cancellation token that can be triggered by Ctrl-C
    let shutdown_token = CancellationToken::new();
    let shutdown_child = shutdown_token.child_token();

    let run_handle = tokio::spawn(async move {
        let mut mgr = manager_run.lock().await;
        let res = mgr.run_all(shutdown_child).await;
        if let Err(e) = &res {
            logger_run.log(LogLevel::Error, &format!("Execution failed: {}", e));
        }
        res
    });

    tokio::select! {
        biased;
        res = run_handle => {
            match res {
                Ok(Ok(())) => {
                    // Normal completion
                }
                Ok(Err(e)) => {
                    logger.log(LogLevel::Error, &format!("Run task error: {}", e));
                }
                Err(join_err) => {
                    logger.log(LogLevel::Error, &format!("Run task join error: {}", join_err));
                }
            }
        }
        _ = tokio::signal::ctrl_c() => {
            logger.log(LogLevel::Info, "Received Ctrl-C, initiating graceful shutdown...");
            // trigger cancellation for running tasks
            shutdown_token.cancel();
            // give components a moment to observe cancellation and stop
            // then perform shutdown_all to cleanup resources
            let mut mgr = manager.lock().await;
            if let Err(e) = mgr.shutdown_all().await {
                logger.log(LogLevel::Error, &format!("Shutdown failed: {}", e));
            } else {
                logger.log(LogLevel::Info, "Shutdown complete");
            }
            return;
        }
    }

    // If run completed on its own, shutdown now
    println!("\n--- Shutdown Phase ---");
    {
        let mut mgr = manager.lock().await;
        if let Err(e) = mgr.shutdown_all().await {
            logger.log(LogLevel::Error, &format!("Shutdown failed: {}", e));
        }
    }

    logger.log(LogLevel::Info, "Application finished");
}

fn main() {
    #[cfg(feature = "tokio_runtime")]
    {
        runtime::run(async_main());
    }

    #[cfg(feature = "rtic_firmware")]
    {
        compile_error!(
            "feature 'rtic_firmware' selected: RTIC firmware builds are provided in a separate crate/scaffold; build for an MCU target"
        );
    }

    #[cfg(not(any(feature = "tokio_runtime", feature = "rtic_firmware")))]
    {
        compile_error!(
            "no runtime feature selected: enable the 'tokio_runtime' feature or select 'rtic_firmware'"
        );
    }
}
