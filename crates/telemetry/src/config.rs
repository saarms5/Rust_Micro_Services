//! Configuration management for telemetry and pipeline
//!
//! Supports loading configuration from:
//! - YAML files (config/telemetry.yaml)
//! - TOML files (config/telemetry.toml)
//! - Environment variables (TELEMETRY_* prefix)
//! - Programmatic defaults

use crate::PipelineConfig;
use crate::resilience::ResilienceConfig;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Configuration error types
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Config file not found: {0}")]
    FileNotFound(String),
    #[error("Failed to parse config: {0}")]
    ParseError(String),
    #[error("Invalid configuration: {0}")]
    Invalid(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Complete telemetry system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryConfig {
    /// Pipeline configuration
    pub pipeline: PipelineConfig,
    /// Resilience configuration
    pub resilience: ResilienceConfig,
    /// Application name
    #[serde(default = "default_app_name")]
    pub app_name: String,
    /// Log level (trace, debug, info, warn, error)
    #[serde(default = "default_log_level")]
    pub log_level: String,
}

fn default_app_name() -> String {
    "rust-telemetry".to_string()
}

fn default_log_level() -> String {
    "info".to_string()
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            pipeline: PipelineConfig::default(),
            resilience: ResilienceConfig::default(),
            app_name: default_app_name(),
            log_level: default_log_level(),
        }
    }
}

/// Configuration loader with precedence: YAML/TOML > Env > Defaults
pub struct ConfigLoader {
    config_dirs: Vec<PathBuf>,
}

impl ConfigLoader {
    /// Create a new configuration loader
    pub fn new() -> Self {
        Self {
            config_dirs: vec![
                PathBuf::from("config"),
                PathBuf::from("."),
                PathBuf::from("/etc/telemetry"),
            ],
        }
    }

    /// Add a configuration directory to search
    pub fn with_config_dir<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.config_dirs.insert(0, path.as_ref().to_path_buf());
        self
    }

    /// Load configuration from files or environment
    pub async fn load(&self) -> Result<TelemetryConfig, ConfigError> {
        // Try to load from YAML or TOML files
        if let Ok(config) = self.load_from_files().await {
            // Override with environment variables
            return Ok(self.apply_env_overrides(config).await);
        }

        // Fall back to environment variables only
        Ok(self.apply_env_overrides(TelemetryConfig::default()).await)
    }

    /// Load from YAML or TOML files
    async fn load_from_files(&self) -> Result<TelemetryConfig, ConfigError> {
        for config_dir in &self.config_dirs {
            // Try YAML first
            let yaml_path = config_dir.join("telemetry.yaml");
            if yaml_path.exists() {
                return self.load_yaml(&yaml_path).await;
            }

            let yaml_alt_path = config_dir.join("telemetry.yml");
            if yaml_alt_path.exists() {
                return self.load_yaml(&yaml_alt_path).await;
            }

            // Try TOML
            let toml_path = config_dir.join("telemetry.toml");
            if toml_path.exists() {
                return self.load_toml(&toml_path).await;
            }
        }

        Err(ConfigError::FileNotFound(
            "No telemetry.yaml or telemetry.toml found".to_string(),
        ))
    }

    /// Load YAML configuration file
    async fn load_yaml(&self, path: &Path) -> Result<TelemetryConfig, ConfigError> {
        let content = tokio::fs::read_to_string(path)
            .await
            .map_err(|e| ConfigError::Io(e))?;

        serde_yaml::from_str(&content)
            .map_err(|e| ConfigError::ParseError(format!("YAML deserialization error: {}", e)))
    }

    /// Load TOML configuration file
    async fn load_toml(&self, path: &Path) -> Result<TelemetryConfig, ConfigError> {
        let content = tokio::fs::read_to_string(path)
            .await
            .map_err(|e| ConfigError::Io(e))?;

        toml::from_str(&content)
            .map_err(|e| ConfigError::ParseError(format!("TOML parse error: {}", e)))
    }

    /// Apply environment variable overrides
    /// Supports: TELEMETRY_PIPELINE_BATCH_SIZE, TELEMETRY_LOG_LEVEL, etc.
    async fn apply_env_overrides(&self, mut config: TelemetryConfig) -> TelemetryConfig {
        // Pipeline overrides
        if let Ok(batch_size) = std::env::var("TELEMETRY_PIPELINE_BATCH_SIZE") {
            if let Ok(size) = batch_size.parse::<usize>() {
                config.pipeline.batch_size = size;
            }
        }

        if let Ok(timeout) = std::env::var("TELEMETRY_PIPELINE_BATCH_TIMEOUT_SECS") {
            if let Ok(secs) = timeout.parse::<u64>() {
                config.pipeline.batch_timeout_secs = secs;
            }
        }

        if let Ok(compression) = std::env::var("TELEMETRY_PIPELINE_ENABLE_COMPRESSION") {
            config.pipeline.enable_compression = compression.to_lowercase() == "true";
        }

        if let Ok(resilience) = std::env::var("TELEMETRY_PIPELINE_ENABLE_RESILIENCE") {
            config.pipeline.enable_resilience = resilience.to_lowercase() == "true";
        }

        if let Ok(capacity) = std::env::var("TELEMETRY_PIPELINE_CHANNEL_CAPACITY") {
            if let Ok(cap) = capacity.parse::<usize>() {
                config.pipeline.channel_capacity = cap;
            }
        }

        // Resilience overrides
        if let Ok(retries) = std::env::var("TELEMETRY_RESILIENCE_MAX_RETRIES") {
            if let Ok(r) = retries.parse::<u32>() {
                config.resilience.max_retries = r;
            }
        }

        if let Ok(threshold) = std::env::var("TELEMETRY_RESILIENCE_FAILURE_THRESHOLD") {
            if let Ok(t) = threshold.parse::<u32>() {
                config.resilience.failure_threshold = t;
            }
        }

        if let Ok(buffer) = std::env::var("TELEMETRY_RESILIENCE_BUFFER_SIZE") {
            if let Ok(size) = buffer.parse::<usize>() {
                config.resilience.buffer_size = size;
            }
        }

        // App overrides
        if let Ok(name) = std::env::var("TELEMETRY_APP_NAME") {
            config.app_name = name;
        }

        if let Ok(level) = std::env::var("TELEMETRY_LOG_LEVEL") {
            config.log_level = level;
        }

        config
    }
}

impl Default for ConfigLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = TelemetryConfig::default();
        assert_eq!(config.app_name, "rust-telemetry");
        assert_eq!(config.log_level, "info");
        assert!(config.pipeline.enable_resilience);
    }

    #[test]
    fn test_config_loader_creation() {
        let loader = ConfigLoader::new();
        assert!(!loader.config_dirs.is_empty());
    }

    #[test]
    fn test_config_with_custom_dir() {
        let loader = ConfigLoader::new().with_config_dir("/custom/path");
        assert_eq!(loader.config_dirs[0], PathBuf::from("/custom/path"));
    }
}
