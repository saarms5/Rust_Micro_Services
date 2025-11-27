//! HAL trait definitions

/// Core trait for hardware abstraction
pub trait HalTrait {
    fn initialize(&mut self) -> Result<(), String>;
    fn shutdown(&mut self) -> Result<(), String>;
    fn health_check(&self) -> Result<bool, String>;
}
