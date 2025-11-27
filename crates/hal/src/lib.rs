//! Hardware Abstraction Layer (HAL)
//! 
//! This crate provides abstractions and interfaces to underlying
//! hardware and system-level components.

pub mod device;
pub mod traits;

pub use device::Device;
pub use traits::HalTrait;
