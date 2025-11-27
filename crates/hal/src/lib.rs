//! Hardware Abstraction Layer (HAL)
//!
//! This crate provides abstractions and interfaces to underlying
//! hardware and system-level components.
//!
//! It uses embedded-hal traits for safe, generic peripheral access
//! and provides register-based wrappers to ensure type-safe MCU interactions.

pub mod device;
pub mod peripherals;
pub mod registers;
pub mod traits;

pub use device::Device;
pub use peripherals::{GpioPin, SpiInterface, TimerUnit, UartPort};
pub use registers::{Register, RegisterValue};
pub use traits::HalTrait;
