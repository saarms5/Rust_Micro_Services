//! Safe register access utilities
//!
//! Provides type-safe wrappers around volatile memory-mapped registers
//! to prevent undefined behavior and ensure correct MCU interactions.

use std::marker::PhantomData;
use std::ptr;

/// Represents a hardware register with read/write access semantics
///
/// This wrapper ensures volatile access to prevent compiler optimizations
/// from caching register reads/writes and provides type-safe bit manipulation.
#[derive(Debug)]
pub struct Register<T: Copy> {
    addr: *mut T,
    // PhantomData ensures the register is tied to its intended use
    _phantom: PhantomData<T>,
}

impl<T: Copy> Register<T> {
    /// Create a new register from a volatile memory location
    ///
    /// # Safety
    ///
    /// The caller must ensure that `addr` points to a valid memory-mapped register
    /// and that the register is not accessed through other means simultaneously.
    pub unsafe fn new(addr: *mut T) -> Self {
        Self {
            addr,
            _phantom: PhantomData,
        }
    }

    /// Read the current value from the register (volatile read)
    pub fn read(&self) -> T {
        unsafe { ptr::read_volatile(self.addr) }
    }

    /// Write a value to the register (volatile write)
    pub fn write(&mut self, value: T) {
        unsafe { ptr::write_volatile(self.addr, value) }
    }

    /// Modify the register by reading, applying a closure, and writing back
    pub fn modify<F>(&mut self, f: F)
    where
        F: FnOnce(T) -> T,
    {
        let value = self.read();
        let modified = f(value);
        self.write(modified);
    }
}

/// Wrapper for register values with bit-level access patterns
#[derive(Debug, Clone, Copy)]
pub struct RegisterValue(pub u32);

impl RegisterValue {
    /// Check if a specific bit is set
    pub fn is_bit_set(&self, bit: u32) -> bool {
        (self.0 & (1 << bit)) != 0
    }

    /// Set a specific bit to 1
    pub fn set_bit(&mut self, bit: u32) {
        self.0 |= 1 << bit;
    }

    /// Clear a specific bit to 0
    pub fn clear_bit(&mut self, bit: u32) {
        self.0 &= !(1 << bit);
    }

    /// Extract a field of bits from [start_bit, end_bit)
    pub fn get_bits(&self, start_bit: u32, end_bit: u32) -> u32 {
        let mask = (1 << (end_bit - start_bit)) - 1;
        (self.0 >> start_bit) & mask
    }

    /// Set a field of bits from [start_bit, end_bit)
    pub fn set_bits(&mut self, start_bit: u32, end_bit: u32, value: u32) {
        let mask = (1 << (end_bit - start_bit)) - 1;
        self.0 = (self.0 & !(mask << start_bit)) | ((value & mask) << start_bit);
    }

    /// Raw value accessor
    pub fn as_u32(&self) -> u32 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_value_bit_operations() {
        let mut val = RegisterValue(0);
        val.set_bit(3);
        assert!(val.is_bit_set(3));
        assert!(!val.is_bit_set(2));

        val.clear_bit(3);
        assert!(!val.is_bit_set(3));
    }

    #[test]
    fn test_register_value_field_operations() {
        let mut val = RegisterValue(0);
        val.set_bits(2, 6, 0b1010); // Set bits [2, 6) to 1010
        assert_eq!(val.get_bits(2, 6), 0b1010);
        assert_eq!(val.as_u32(), 0b101000);
    }
}
