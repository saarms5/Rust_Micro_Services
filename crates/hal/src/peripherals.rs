//! Peripheral implementations
//!
//! Provides safe abstractions for common MCU peripherals:
//! GPIO, UART, SPI, and Timer.
//!
//! These implementations focus on type-safety and preventing common errors
//! like accessing uninitialized peripherals or using invalid configurations.

/// GPIO Pin abstraction
///
/// Safely manages digital output pins without exposing register details
/// to the application layer.
#[derive(Debug, Clone, Copy)]
pub struct GpioPin {
    pin_num: u8,
    is_high: bool,
}

impl GpioPin {
    /// Create a new GPIO pin
    pub fn new(pin_num: u8) -> Self {
        Self {
            pin_num,
            is_high: false,
        }
    }

    /// Get the pin number
    pub fn pin_num(&self) -> u8 {
        self.pin_num
    }

    /// Check if pin is currently high
    pub fn is_high(&self) -> bool {
        self.is_high
    }

    /// Set pin to LOW
    pub fn set_low(&mut self) -> Result<(), String> {
        println!("[GPIO {}] Setting pin LOW", self.pin_num);
        self.is_high = false;
        Ok(())
    }

    /// Set pin to HIGH
    pub fn set_high(&mut self) -> Result<(), String> {
        println!("[GPIO {}] Setting pin HIGH", self.pin_num);
        self.is_high = true;
        Ok(())
    }

    /// Toggle the pin state
    pub fn toggle(&mut self) -> Result<(), String> {
        self.is_high = !self.is_high;
        println!(
            "[GPIO {}] Toggled to {}",
            self.pin_num,
            if self.is_high { "HIGH" } else { "LOW" }
        );
        Ok(())
    }
}

/// UART Port abstraction for serial communication
///
/// Provides safe serial communication without exposing register details
#[derive(Debug, Clone)]
pub struct UartPort {
    port_num: u8,
    baud_rate: u32,
    is_open: bool,
}

impl UartPort {
    /// Create a new UART port
    pub fn new(port_num: u8, baud_rate: u32) -> Self {
        Self {
            port_num,
            baud_rate,
            is_open: false,
        }
    }

    /// Open the UART port for communication
    pub fn open(&mut self) -> Result<(), String> {
        println!(
            "[UART {}] Opening at {} baud",
            self.port_num, self.baud_rate
        );
        self.is_open = true;
        Ok(())
    }

    /// Close the UART port
    pub fn close(&mut self) -> Result<(), String> {
        println!("[UART {}] Closing", self.port_num);
        self.is_open = false;
        Ok(())
    }

    /// Write data to the UART port
    pub fn write(&self, data: &[u8]) -> Result<(), String> {
        if !self.is_open {
            return Err("UART port not open".to_string());
        }
        println!(
            "[UART {}] Writing {} bytes",
            self.port_num,
            data.len()
        );
        Ok(())
    }

    /// Read data from the UART port
    pub fn read(&self) -> Result<Vec<u8>, String> {
        if !self.is_open {
            return Err("UART port not open".to_string());
        }
        println!("[UART {}] Reading data", self.port_num);
        Ok(vec![])
    }
}

/// SPI Interface abstraction
///
/// Provides safe SPI communication for sensors and peripherals
#[derive(Debug, Clone)]
pub struct SpiInterface {
    bus_num: u8,
    clock_speed: u32,
    is_active: bool,
}

impl SpiInterface {
    /// Create a new SPI interface
    pub fn new(bus_num: u8, clock_speed: u32) -> Self {
        Self {
            bus_num,
            clock_speed,
            is_active: false,
        }
    }

    /// Initialize the SPI bus
    pub fn initialize(&mut self) -> Result<(), String> {
        println!(
            "[SPI {}] Initializing at {} Hz",
            self.bus_num, self.clock_speed
        );
        self.is_active = true;
        Ok(())
    }

    /// Deinitialize the SPI bus
    pub fn deinitialize(&mut self) -> Result<(), String> {
        println!("[SPI {}] Deinitializing", self.bus_num);
        self.is_active = false;
        Ok(())
    }

    /// Transfer data over SPI
    pub fn transfer(&self, tx_data: &[u8]) -> Result<Vec<u8>, String> {
        if !self.is_active {
            return Err("SPI bus not initialized".to_string());
        }
        println!("[SPI {}] Transferring {} bytes", self.bus_num, tx_data.len());
        // Echo back the data for demonstration
        Ok(tx_data.to_vec())
    }
}

/// Timer Unit abstraction
///
/// Provides safe timer/counter access for delays and PWM
#[derive(Debug, Clone, Copy)]
pub struct TimerUnit {
    timer_num: u8,
    prescaler: u32,
    is_running: bool,
}

impl TimerUnit {
    /// Create a new timer unit
    pub fn new(timer_num: u8, prescaler: u32) -> Self {
        Self {
            timer_num,
            prescaler,
            is_running: false,
        }
    }

    /// Start the timer
    pub fn start(&mut self) -> Result<(), String> {
        println!(
            "[Timer {}] Starting with prescaler {}",
            self.timer_num, self.prescaler
        );
        self.is_running = true;
        Ok(())
    }

    /// Stop the timer
    pub fn stop(&mut self) -> Result<(), String> {
        println!("[Timer {}] Stopping", self.timer_num);
        self.is_running = false;
        Ok(())
    }

    /// Set the timer interval in milliseconds
    pub fn set_interval_ms(&mut self, ms: u32) -> Result<(), String> {
        if !self.is_running {
            return Err("Timer not running".to_string());
        }
        println!("[Timer {}] Set interval to {} ms", self.timer_num, ms);
        Ok(())
    }

    /// Check if timer is running
    pub fn is_running(&self) -> bool {
        self.is_running
    }
}
