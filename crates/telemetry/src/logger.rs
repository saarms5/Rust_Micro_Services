//! Logging utilities

/// Simple logger implementation
#[derive(Debug)]
pub struct Logger {
    #[allow(dead_code)]
    level: LogLevel,
}

#[derive(Debug, Clone, Copy)]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
}

impl Logger {
    pub fn new(level: LogLevel) -> Self {
        Self { level }
    }

    pub fn log(&self, level: LogLevel, message: &str) {
        println!("[{:?}] {}", level, message);
    }
}
