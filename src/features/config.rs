//! Configuration management for NEX2426
//! 
//! Provides structured configuration handling with support for
//! multiple formats and validation.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NexConfig {
    /// Default cost parameter for operations
    pub default_cost: u32,
    /// Enable temporal binding by default
    pub temporal_binding: bool,
    /// Default output format
    pub output_format: OutputFormat,
    /// Logging configuration
    pub logging: LoggingConfig,
    /// Performance settings
    pub performance: PerformanceConfig,
    /// Security settings
    pub security: SecurityConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutputFormat {
    /// Standard nex6 format
    Standard,
    /// Base64 encoded
    Base64,
    /// Hexadecimal
    Hex,
    /// JSON format
    Json,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Enable logging
    pub enabled: bool,
    /// Log level (trace, debug, info, warn, error)
    pub level: String,
    /// Log file path (optional, defaults to stdout)
    pub file: Option<String>,
    /// Enable performance metrics
    pub metrics: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Number of worker threads (0 = auto)
    pub worker_threads: usize,
    /// Enable parallel processing
    pub parallel: bool,
    /// Memory limit in MB
    pub memory_limit_mb: Option<usize>,
    /// Cache size for operations
    pub cache_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Enable hardware binding by default
    pub hardware_binding: bool,
    /// Require secure memory
    pub secure_memory: bool,
    /// Enable audit logging
    pub audit_logging: bool,
    /// Maximum allowed cost parameter
    pub max_cost: u32,
}

impl Default for NexConfig {
    fn default() -> Self {
        Self {
            default_cost: 3,
            temporal_binding: false,
            output_format: OutputFormat::Standard,
            logging: LoggingConfig {
                enabled: false,
                level: "info".to_string(),
                file: None,
                metrics: false,
            },
            performance: PerformanceConfig {
                worker_threads: 0, // Auto-detect
                parallel: true,
                memory_limit_mb: Some(1024),
                cache_size: 1000,
            },
            security: SecurityConfig {
                hardware_binding: false,
                secure_memory: false,
                audit_logging: false,
                max_cost: 10,
            },
        }
    }
}

impl NexConfig {
    /// Load configuration from file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let path_ref = path.as_ref();
        let content = fs::read_to_string(path_ref)?;
        
        // Try to detect format based on file extension
        let config = if let Some(ext) = path_ref.extension() {
            match ext.to_str().unwrap() {
                "json" => serde_json::from_str(&content)?,
                "toml" => toml::from_str(&content)?,
                "yaml" | "yml" => serde_yaml::from_str(&content)?,
                _ => return Err("Unsupported configuration file format".into()),
            }
        } else {
            return Err("Cannot determine configuration file format".into());
        };
        
        Ok(config)
    }
    
    /// Save configuration to file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let path_ref = path.as_ref();
        
        let content = if let Some(ext) = path_ref.extension() {
            match ext.to_str().unwrap() {
                "json" => serde_json::to_string_pretty(self)?,
                "toml" => toml::to_string_pretty(self)?,
                "yaml" | "yml" => serde_yaml::to_string(self)?,
                _ => return Err("Unsupported configuration file format".into()),
            }
        } else {
            return Err("Cannot determine configuration file format".into());
        };
        
        fs::write(path, content)?;
        Ok(())
    }
    
    /// Load configuration from environment variables
    pub fn load_from_env() -> Self {
        let mut config = Self::default();
        
        // Override with environment variables
        if let Ok(cost) = std::env::var("NEX_COST") {
            if let Ok(cost_val) = cost.parse() {
                config.default_cost = cost_val;
            }
        }
        
        if let Ok(temporal) = std::env::var("NEX_TEMPORAL") {
            config.temporal_binding = temporal.parse().unwrap_or(false);
        }
        
        if let Ok(level) = std::env::var("NEX_LOG_LEVEL") {
            config.logging.level = level;
        }
        
        if let Ok(enabled) = std::env::var("NEX_LOG_ENABLED") {
            config.logging.enabled = enabled.parse().unwrap_or(false);
        }
        
        if let Ok(threads) = std::env::var("NEX_WORKER_THREADS") {
            if let Ok(threads_val) = threads.parse() {
                config.performance.worker_threads = threads_val;
            }
        }
        
        config
    }
    
    /// Validate configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.default_cost == 0 || self.default_cost > self.security.max_cost {
            return Err(format!(
                "Default cost must be between 1 and {}", 
                self.security.max_cost
            ));
        }
        
        if self.performance.worker_threads > 64 {
            return Err("Worker threads should not exceed 64".to_string());
        }
        
        if self.performance.cache_size == 0 {
            return Err("Cache size must be greater than 0".to_string());
        }
        
        Ok(())
    }
    
    /// Get effective number of worker threads
    pub fn worker_threads(&self) -> usize {
        if self.performance.worker_threads == 0 {
            num_cpus::get()
        } else {
            self.performance.worker_threads
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_default_config() {
        let config = NexConfig::default();
        assert_eq!(config.default_cost, 3);
        assert!(!config.temporal_binding);
        assert!(config.validate().is_ok());
    }
    
    #[test]
    fn test_config_validation() {
        let mut config = NexConfig::default();
        
        // Valid config
        assert!(config.validate().is_ok());
        
        // Invalid cost
        config.default_cost = 0;
        assert!(config.validate().is_err());
        
        config.default_cost = 100;
        assert!(config.validate().is_err());
    }
    
    #[test]
    fn test_json_config() {
        let config = NexConfig::default();
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("config.json");
        
        // Save config
        config.save_to_file(&file_path).unwrap();
        
        // Load config
        let loaded_config = NexConfig::load_from_file(&file_path).unwrap();
        assert_eq!(config.default_cost, loaded_config.default_cost);
    }
    
    #[test]
    fn test_env_override() {
        std::env::set_var("NEX_COST", "5");
        std::env::set_var("NEX_TEMPORAL", "true");
        
        let config = NexConfig::load_from_env();
        assert_eq!(config.default_cost, 5);
        assert!(config.temporal_binding);
        
        // Clean up
        std::env::remove_var("NEX_COST");
        std::env::remove_var("NEX_TEMPORAL");
    }
}
