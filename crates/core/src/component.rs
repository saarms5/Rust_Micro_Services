//! Component trait definitions for standardized lifecycle management

use async_trait::async_trait;
use tokio_util::sync::CancellationToken;

/// Error type for component operations
#[derive(Debug, Clone)]
pub struct ComponentError {
    pub message: String,
}

impl ComponentError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for ComponentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Component Error: {}", self.message)
    }
}

impl std::error::Error for ComponentError {}

pub type ComponentResult<T> = Result<T, ComponentError>;

/// Trait for standardizing component lifecycle and behavior
///
/// Sensors, actuators, and other components should implement this trait
/// to provide consistent initialization, execution, and shutdown behavior.
#[async_trait]
pub trait Component: Send + Sync {
    /// Get the unique identifier for this component
    fn id(&self) -> &str;

    /// Get a human-readable name for this component
    fn name(&self) -> &str;

    /// Initialize the component
    ///
    /// Called once during startup. Should set up any required resources,
    /// perform hardware initialization, establish connections, etc.
    async fn init(&mut self) -> ComponentResult<()>;

    /// Run the component's main logic
    ///
    /// Called after initialization to perform the component's primary function.
    /// This may run in a loop or block until completion/shutdown. A
    /// `CancellationToken` is provided so the runtime can request cancellation
    /// (for example on Ctrl-C) and components can stop early.
    async fn run(&mut self, shutdown: CancellationToken) -> ComponentResult<()>;

    /// Shutdown the component gracefully
    ///
    /// Called during application shutdown. Should clean up resources,
    /// close connections, and prepare for termination.
    async fn shutdown(&mut self) -> ComponentResult<()>;

    /// Get the current health status of the component
    ///
    /// Returns Ok(()) if healthy, or an error describing the issue
    async fn health_check(&self) -> ComponentResult<()>;

    /// Optional: Configure the component before initialization
    ///
    /// Default implementation does nothing
    fn configure(&mut self, _config: &str) -> ComponentResult<()> {
        Ok(())
    }
}

/// A manager for handling multiple components
pub struct ComponentManager {
    components: Vec<Box<dyn Component>>,
}

impl ComponentManager {
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
        }
    }

    pub fn register(&mut self, component: Box<dyn Component>) {
        self.components.push(component);
    }

    pub async fn init_all(&mut self) -> ComponentResult<()> {
        for component in &mut self.components {
            eprintln!("Initializing component: {}", component.name());
            component.init().await?;
        }
        Ok(())
    }

    /// Run all components, passing each a clone of the provided `CancellationToken`.
    pub async fn run_all(&mut self, shutdown: CancellationToken) -> ComponentResult<()> {
        for component in &mut self.components {
            eprintln!("Running component: {}", component.name());
            component.run(shutdown.clone()).await?;
        }
        Ok(())
    }

    pub async fn shutdown_all(&mut self) -> ComponentResult<()> {
        // Shutdown in reverse order
        for component in self.components.iter_mut().rev() {
            eprintln!("Shutting down component: {}", component.name());
            component.shutdown().await?;
        }
        Ok(())
    }

    pub async fn health_check_all(&self) -> ComponentResult<()> {
        for component in &self.components {
            component.health_check().await?;
        }
        Ok(())
    }
}

impl Default for ComponentManager {
    fn default() -> Self {
        Self::new()
    }
}
