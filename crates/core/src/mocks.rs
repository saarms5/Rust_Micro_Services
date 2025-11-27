//! Mock sensor implementations for development and testing without hardware
//!
//! These sensors simulate realistic data patterns to enable rapid development
//! and testing of application logic before integrating real hardware.

use crate::component::{Component, ComponentResult};
use async_trait::async_trait;
use tokio_util::sync::CancellationToken;

/// Mock GPS sensor that generates synthetic coordinates
///
/// Simulates a GPS module by incrementally updating latitude/longitude
/// and varying the satellite count and accuracy.
#[derive(Debug)]
pub struct MockGpsSensor {
    id: String,
    name: String,
    latitude: f64,
    longitude: f64,
    altitude: f64,
    satellites: u8,
    accuracy: f32,
    is_initialized: bool,
}

impl MockGpsSensor {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            latitude: 37.7749, // San Francisco
            longitude: -122.4194,
            altitude: 0.0,
            satellites: 0,
            accuracy: 0.0,
            is_initialized: false,
        }
    }
}

#[async_trait]
impl Component for MockGpsSensor {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    async fn init(&mut self) -> ComponentResult<()> {
        println!("[{}] Initializing GPS sensor...", self.name);
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        self.is_initialized = true;
        self.satellites = 4;
        self.accuracy = 5.0;
        println!("[{}] GPS initialized, waiting for signal...", self.name);
        Ok(())
    }

    async fn run(&mut self, shutdown: CancellationToken) -> ComponentResult<()> {
        if !self.is_initialized {
            return Err(crate::component::ComponentError::new("GPS not initialized"));
        }

        println!("[{}] Running GPS acquisition...", self.name);
        let mut iteration = 0;

        loop {
            tokio::select! {
                _ = shutdown.cancelled() => {
                    println!("[{}] Shutdown requested, stopping GPS", self.name);
                    return Ok(());
                }
                _ = tokio::time::sleep(tokio::time::Duration::from_millis(500)) => {
                    iteration += 1;

                    // Simulate gradual satellite acquisition
                    if iteration < 3 {
                        self.satellites = (4 + iteration as u8).min(12);
                    }

                    // Improve accuracy as satellites lock on
                    self.accuracy = (5.0 - (iteration as f32 * 0.5)).max(0.5);

                    // Simulate slow drift in position
                    self.latitude += 0.00001 * (iteration as f64 % 5.0 - 2.0);
                    self.longitude -= 0.00001 * (iteration as f64 % 3.0 - 1.5);
                    self.altitude = 100.0 + (iteration as f64 * 0.5) % 50.0;

                    println!(
                        "[{}] Fix: Lat {:.4}°, Lon {:.4}°, Alt {:.1}m, Sats {}, Acc {:.1}m",
                        self.name, self.latitude, self.longitude, self.altitude, self.satellites, self.accuracy
                    );

                    if iteration >= 10 {
                        break;
                    }
                }
            }
        }

        println!("[{}] GPS acquisition complete", self.name);
        Ok(())
    }

    async fn shutdown(&mut self) -> ComponentResult<()> {
        println!("[{}] Shutting down GPS sensor...", self.name);
        self.is_initialized = false;
        Ok(())
    }

    async fn health_check(&self) -> ComponentResult<()> {
        if !self.is_initialized {
            return Err(crate::component::ComponentError::new("GPS not initialized"));
        }
        if self.satellites < 4 {
            return Err(crate::component::ComponentError::new(
                "Insufficient satellite lock",
            ));
        }
        Ok(())
    }
}

/// Mock IMU (Inertial Measurement Unit) sensor
///
/// Simulates accelerometer, gyroscope, and magnetometer readings with realistic noise patterns.
#[derive(Debug)]
pub struct MockImuSensor {
    id: String,
    name: String,
    accel_x: f32,
    accel_y: f32,
    accel_z: f32,
    gyro_x: f32,
    gyro_y: f32,
    gyro_z: f32,
    #[allow(dead_code)]
    mag_x: f32,
    #[allow(dead_code)]
    mag_y: f32,
    #[allow(dead_code)]
    mag_z: f32,
    temperature: f32,
    is_initialized: bool,
}

impl MockImuSensor {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            accel_x: 0.0,
            accel_y: 0.0,
            accel_z: 9.81, // 1G downward
            gyro_x: 0.0,
            gyro_y: 0.0,
            gyro_z: 0.0,
            mag_x: 20.0,
            mag_y: 0.0,
            mag_z: 40.0,
            temperature: 25.0,
            is_initialized: false,
        }
    }
}

#[async_trait]
impl Component for MockImuSensor {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    async fn init(&mut self) -> ComponentResult<()> {
        println!("[{}] Initializing IMU sensor...", self.name);
        tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;
        self.is_initialized = true;
        println!("[{}] IMU initialized", self.name);
        Ok(())
    }

    async fn run(&mut self, shutdown: CancellationToken) -> ComponentResult<()> {
        if !self.is_initialized {
            return Err(crate::component::ComponentError::new("IMU not initialized"));
        }

        println!("[{}] Collecting IMU data...", self.name);
        let mut iteration = 0;

        loop {
            tokio::select! {
                _ = shutdown.cancelled() => {
                    println!("[{}] Shutdown requested, stopping IMU", self.name);
                    return Ok(());
                }
                _ = tokio::time::sleep(tokio::time::Duration::from_millis(300)) => {
                    iteration += 1;

                    // Simulate motion: gradual rotation
                    self.gyro_x = (iteration as f32 * 0.5).sin() * 10.0; // ±10 deg/s
                    self.gyro_y = (iteration as f32 * 0.3).cos() * 5.0;  // ±5 deg/s
                    self.gyro_z = 0.0;

                    // Simulate acceleration from motion
                    self.accel_x = (iteration as f32 * 0.2).sin() * 2.0; // ±2 m/s²
                    self.accel_y = (iteration as f32 * 0.1).cos() * 1.5; // ±1.5 m/s²
                    self.accel_z = 9.81 + (iteration as f32 * 0.1).sin() * 0.5;

                    // Simulate temperature drift
                    self.temperature = 25.0 + (iteration as f32 * 0.05);

                    println!(
                        "[{}] Accel: [{:6.2}, {:6.2}, {:6.2}] m/s² | Gyro: [{:6.1}, {:6.1}, {:6.1}] °/s | Temp: {:.1}°C",
                        self.name,
                        self.accel_x, self.accel_y, self.accel_z,
                        self.gyro_x, self.gyro_y, self.gyro_z,
                        self.temperature
                    );

                    if iteration >= 8 {
                        break;
                    }
                }
            }
        }

        println!("[{}] IMU data collection complete", self.name);
        Ok(())
    }

    async fn shutdown(&mut self) -> ComponentResult<()> {
        println!("[{}] Shutting down IMU sensor...", self.name);
        self.is_initialized = false;
        Ok(())
    }

    async fn health_check(&self) -> ComponentResult<()> {
        if !self.is_initialized {
            return Err(crate::component::ComponentError::new("IMU not initialized"));
        }
        // Check temperature is within operating range
        if self.temperature < -40.0 || self.temperature > 85.0 {
            return Err(crate::component::ComponentError::new(
                "IMU temperature out of range",
            ));
        }
        Ok(())
    }
}

/// Mock Barometer sensor
///
/// Simulates atmospheric pressure, temperature, and altitude measurements
/// with realistic variations.
#[derive(Debug)]
pub struct MockBarometerSensor {
    id: String,
    name: String,
    pressure: f32,    // in hPa
    temperature: f32, // in °C
    altitude: f32,    // in meters
    is_initialized: bool,
}

impl MockBarometerSensor {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            pressure: 1013.25, // Standard sea level pressure
            temperature: 15.0,
            altitude: 0.0,
            is_initialized: false,
        }
    }
}

#[async_trait]
impl Component for MockBarometerSensor {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    async fn init(&mut self) -> ComponentResult<()> {
        println!("[{}] Initializing Barometer sensor...", self.name);
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        self.is_initialized = true;
        println!("[{}] Barometer initialized", self.name);
        Ok(())
    }

    async fn run(&mut self, shutdown: CancellationToken) -> ComponentResult<()> {
        if !self.is_initialized {
            return Err(crate::component::ComponentError::new(
                "Barometer not initialized",
            ));
        }

        println!("[{}] Reading atmospheric data...", self.name);
        let mut iteration = 0;

        loop {
            tokio::select! {
                _ = shutdown.cancelled() => {
                    println!("[{}] Shutdown requested, stopping Barometer", self.name);
                    return Ok(());
                }
                _ = tokio::time::sleep(tokio::time::Duration::from_millis(600)) => {
                    iteration += 1;

                    // Simulate gradual climb: altitude increases, pressure decreases
                    self.altitude = iteration as f32 * 5.0; // 5m per reading
                    // Pressure decreases ~12 Pa per 100m
                    self.pressure = 1013.25 - (self.altitude * 0.12);

                    // Temperature decreases ~6.5°C per 1000m
                    self.temperature = 15.0 - (self.altitude * 0.0065);

                    println!(
                        "[{}] Pressure: {:.2} hPa, Temp: {:.1}°C, Altitude: {:.1}m",
                        self.name, self.pressure, self.temperature, self.altitude
                    );

                    if iteration >= 6 {
                        break;
                    }
                }
            }
        }

        println!("[{}] Atmospheric measurement complete", self.name);
        Ok(())
    }

    async fn shutdown(&mut self) -> ComponentResult<()> {
        println!("[{}] Shutting down Barometer sensor...", self.name);
        self.is_initialized = false;
        Ok(())
    }

    async fn health_check(&self) -> ComponentResult<()> {
        if !self.is_initialized {
            return Err(crate::component::ComponentError::new(
                "Barometer not initialized",
            ));
        }
        if self.pressure < 300.0 || self.pressure > 1100.0 {
            return Err(crate::component::ComponentError::new(
                "Barometer pressure out of range",
            ));
        }
        Ok(())
    }
}
