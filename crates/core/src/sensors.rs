//! Example sensor component implementation

use crate::component::{Component, ComponentResult};
use async_trait::async_trait;
use tokio_util::sync::CancellationToken;

/// Example temperature sensor component
#[derive(Debug)]
pub struct TemperatureSensor {
    id: String,
    name: String,
    current_value: f32,
    is_initialized: bool,
}

impl TemperatureSensor {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            current_value: 20.0,
            is_initialized: false,
        }
    }
}

#[async_trait]
impl Component for TemperatureSensor {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    async fn init(&mut self) -> ComponentResult<()> {
        println!("[{}] Initializing sensor...", self.name);
        // Simulate hardware initialization
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        self.is_initialized = true;
        self.current_value = 22.5;
        println!("[{}] Sensor initialized successfully", self.name);
        Ok(())
    }

    async fn run(&mut self, shutdown: CancellationToken) -> ComponentResult<()> {
        if !self.is_initialized {
            return Err(crate::component::ComponentError::new("Sensor not initialized"));
        }

        println!("[{}] Running sensor loop...", self.name);
        // Simulate sensor reading; react to shutdown token
        for i in 0..5 {
            tokio::select! {
                _ = shutdown.cancelled() => {
                    println!("[{}] Shutdown requested, stopping sensor loop", self.name);
                    return Ok(());
                }
                _ = tokio::time::sleep(tokio::time::Duration::from_millis(200)) => {
                    self.current_value += 0.5;
                    println!("[{}] Reading {}: {:.1}°C", self.name, i + 1, self.current_value);
                }
            }
        }
        Ok(())
    }

    async fn shutdown(&mut self) -> ComponentResult<()> {
        println!("[{}] Shutting down sensor...", self.name);
        self.is_initialized = false;
        Ok(())
    }

    async fn health_check(&self) -> ComponentResult<()> {
        if !self.is_initialized {
            return Err(crate::component::ComponentError::new("Sensor not initialized"));
        }
        if self.current_value > 100.0 || self.current_value < -50.0 {
            return Err(crate::component::ComponentError::new("Temperature out of range"));
        }
        Ok(())
    }
}

/// Example actuator component
#[derive(Debug)]
pub struct MotorActuator {
    id: String,
    name: String,
    is_running: bool,
    is_initialized: bool,
}

impl MotorActuator {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            is_running: false,
            is_initialized: false,
        }
    }
}

#[async_trait]
impl Component for MotorActuator {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    async fn init(&mut self) -> ComponentResult<()> {
        println!("[{}] Initializing motor...", self.name);
        tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;
        self.is_initialized = true;
        println!("[{}] Motor initialized successfully", self.name);
        Ok(())
    }

    async fn run(&mut self, shutdown: CancellationToken) -> ComponentResult<()> {
        if !self.is_initialized {
            return Err(crate::component::ComponentError::new("Motor not initialized"));
        }

        println!("[{}] Starting motor...", self.name);
        self.is_running = true;

        for speed in (0..=100).step_by(20) {
            tokio::select! {
                _ = shutdown.cancelled() => {
                    println!("[{}] Shutdown requested, stopping motor...", self.name);
                    self.is_running = false;
                    return Ok(());
                }
                _ = tokio::time::sleep(tokio::time::Duration::from_millis(300)) => {
                    println!("[{}] Motor speed: {}%", self.name, speed);
                }
            }
        }

        println!("[{}] Stopping motor...", self.name);
        self.is_running = false;
        Ok(())
    }

    async fn shutdown(&mut self) -> ComponentResult<()> {
        println!("[{}] Shutting down motor...", self.name);
        if self.is_running {
            self.is_running = false;
        }
        self.is_initialized = false;
        Ok(())
    }

    async fn health_check(&self) -> ComponentResult<()> {
        if !self.is_initialized {
            return Err(crate::component::ComponentError::new("Motor not initialized"));
        }
        Ok(())
    }
}
