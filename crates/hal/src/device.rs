//! Device abstraction implementations

use crate::traits::HalTrait;

/// A basic hardware device abstraction
#[derive(Debug)]
pub struct Device {
    pub name: String,
    initialized: bool,
}

impl Device {
    pub fn new(name: String) -> Self {
        Self {
            name,
            initialized: false,
        }
    }
}

impl HalTrait for Device {
    fn initialize(&mut self) -> Result<(), String> {
        self.initialized = true;
        Ok(())
    }

    fn shutdown(&mut self) -> Result<(), String> {
        self.initialized = false;
        Ok(())
    }

    fn health_check(&self) -> Result<bool, String> {
        Ok(self.initialized)
    }
}
