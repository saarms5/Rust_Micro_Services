//! Metrics collection and reporting

use std::collections::HashMap;

/// Simple metrics collector
#[derive(Debug)]
pub struct Metrics {
    counters: HashMap<String, u64>,
}

impl Metrics {
    pub fn new() -> Self {
        Self {
            counters: HashMap::new(),
        }
    }

    pub fn increment(&mut self, name: &str) {
        *self.counters.entry(name.to_string()).or_insert(0) += 1;
    }

    pub fn get(&self, name: &str) -> Option<u64> {
        self.counters.get(name).copied()
    }

    pub fn report(&self) {
        for (name, value) in &self.counters {
            println!("Metric {}: {}", name, value);
        }
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}
