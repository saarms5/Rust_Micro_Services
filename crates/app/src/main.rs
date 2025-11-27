use core::{ComponentManager, MotorActuator, TemperatureSensor};
use telemetry::{Logger, LogLevel};

#[tokio::main]
async fn main() {
    println!("=== Rust Microservices Application ===\n");

    let logger = Logger::new(LogLevel::Info);
    logger.log(LogLevel::Info, "Application starting...");

    // Create component manager
    let mut manager = ComponentManager::new();

    // Register components
    let sensor = Box::new(TemperatureSensor::new("sensor-001", "Temperature Sensor"));
    let motor = Box::new(MotorActuator::new("motor-001", "Main Motor"));

    manager.register(sensor);
    manager.register(motor);

    // Initialize all components
    println!("\n--- Initialization Phase ---");
    if let Err(e) = manager.init_all().await {
        logger.log(LogLevel::Error, &format!("Initialization failed: {}", e));
        return;
    }

    // Health check
    println!("\n--- Health Check Phase ---");
    if let Err(e) = manager.health_check_all().await {
        logger.log(LogLevel::Error, &format!("Health check failed: {}", e));
    } else {
        logger.log(LogLevel::Info, "All components healthy");
    }

    // Run all components
    println!("\n--- Execution Phase ---");
    if let Err(e) = manager.run_all().await {
        logger.log(LogLevel::Error, &format!("Execution failed: {}", e));
    }

    // Shutdown all components
    println!("\n--- Shutdown Phase ---");
    if let Err(e) = manager.shutdown_all().await {
        logger.log(LogLevel::Error, &format!("Shutdown failed: {}", e));
    }

    logger.log(LogLevel::Info, "Application finished");
}
