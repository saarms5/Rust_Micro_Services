//! Core logic module
//! 
//! This crate contains the core business logic and domain models
//! for the microservices application.

pub mod component;
pub mod models;
pub mod sensors;

pub use component::{Component, ComponentError, ComponentManager, ComponentResult};
pub use sensors::{MotorActuator, TemperatureSensor};

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
