//! Core logic module
//! 
//! This crate contains the core business logic and domain models
//! for the microservices application.

pub mod component;
pub mod models;
pub mod sensors;
pub mod scheduler;
pub mod control_loops;

#[cfg(feature = "mock_sensors")]
pub mod mocks;

pub use component::{Component, ComponentError, ComponentManager, ComponentResult};
pub use sensors::{MotorActuator, TemperatureSensor};
pub use scheduler::{ControlLoopTask, RealTimeLoop, MixedPriorityRuntime, SchedulerResult, SchedulerError};
pub use control_loops::{ExampleControlLoop, PidControlLoop};

#[cfg(feature = "mock_sensors")]
pub use mocks::{MockBarometerSensor, MockGpsSensor, MockImuSensor};

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
