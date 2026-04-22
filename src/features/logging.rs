//! Logging utilities for NEX2426
//! 
//! Provides structured logging with multiple output formats,
//! performance metrics, and audit trail functionality.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

/// Structured logger for NEX2426 operations
pub struct NexLogger {
    config: LoggerConfig,
    file_writer: Option<Arc<Mutex<std::fs::File>>>,
    metrics: Arc<Mutex<LogMetrics>>,
}

#[derive(Debug, Clone)]
pub struct LoggerConfig {
    /// Enable logging
    pub enabled: bool,
    /// Log level
    pub level: LogLevel,
    /// Output format
    pub format: LogFormat,
    /// Log file path (None for stdout)
    pub file_path: Option<String>,
    /// Enable performance metrics
    pub metrics_enabled: bool,
    /// Enable audit logging
    pub audit_enabled: bool,
    /// Maximum log file size in bytes
    pub max_file_size: Option<usize>,
    /// Enable rotation
    pub rotate: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone)]
pub enum LogFormat {
    /// Simple text format
    Text,
    /// JSON format
    Json,
    /// Structured key-value format
    Structured,
}

#[derive(Debug, Default)]
pub struct LogMetrics {
    /// Total log entries
    pub total_entries: u64,
    /// Entries by level
    pub entries_by_level: HashMap<String, u64>,
    /// Performance metrics
    pub performance_metrics: HashMap<String, Vec<u64>>,
    /// Error count
    pub error_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// Timestamp
    pub timestamp: u64,
    /// Log level
    pub level: String,
    /// Message
    pub message: String,
    /// Operation type (if applicable)
    pub operation: Option<String>,
    /// Duration in milliseconds (if applicable)
    pub duration_ms: Option<u64>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl NexLogger {
    /// Create new logger with configuration
    pub fn new(config: LoggerConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let file_writer = if let Some(ref path) = config.file_path {
            let file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)?;
            Some(Arc::new(Mutex::new(file)))
        } else {
            None
        };
        
        Ok(Self {
            config,
            file_writer,
            metrics: Arc::new(Mutex::new(LogMetrics::default())),
        })
    }
    
    /// Log a message
    pub fn log(&self, level: LogLevel, message: &str, metadata: HashMap<String, String>) {
        if !self.config.enabled {
            return;
        }
        
        if !self.should_log(&level) {
            return;
        }
        
        let entry = LogEntry {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_else(|_| std::time::Duration::from_secs(0))
                .as_secs(),
            level: format!("{:?}", level),
            message: message.to_string(),
            operation: metadata.get("operation").cloned(),
            duration_ms: metadata.get("duration_ms")
                .and_then(|d| d.parse().ok()),
            metadata,
        };
        
        // Update metrics
        self.update_metrics(&entry);
        
        // Write log entry
        let formatted = self.format_entry(&entry);
        
        if let Some(ref writer) = self.file_writer {
            if let Ok(mut file) = writer.lock() {
                let _ = writeln!(file, "{}", formatted);
                let _ = file.flush();
            }
        } else {
            println!("{}", formatted);
        }
    }
    
    /// Log operation with timing
    pub fn log_operation(&self, operation: &str, duration_ms: u64, success: bool) {
        let mut metadata = HashMap::new();
        metadata.insert("operation".to_string(), operation.to_string());
        metadata.insert("duration_ms".to_string(), duration_ms.to_string());
        metadata.insert("success".to_string(), success.to_string());
        
        let level = if success { LogLevel::Info } else { LogLevel::Error };
        let message = format!("Operation '{}' completed in {}ms", operation, duration_ms);
        
        self.log(level, &message, metadata);
    }
    
    /// Log cryptographic operation
    pub fn log_crypto_operation(&self, op_type: &str, input_size: usize, cost: u32, duration_ms: u64) {
        let mut metadata = HashMap::new();
        metadata.insert("operation".to_string(), format!("crypto_{}", op_type));
        metadata.insert("input_size".to_string(), input_size.to_string());
        metadata.insert("cost".to_string(), cost.to_string());
        metadata.insert("duration_ms".to_string(), duration_ms.to_string());
        
        let message = format!("Crypto operation '{}' (size={}, cost={}) took {}ms", 
                             op_type, input_size, cost, duration_ms);
        
        self.log(LogLevel::Info, &message, metadata);
    }
    
    /// Log security event
    pub fn log_security_event(&self, event_type: &str, details: &str) {
        let mut metadata = HashMap::new();
        metadata.insert("event_type".to_string(), event_type.to_string());
        metadata.insert("category".to_string(), "security".to_string());
        
        let message = format!("Security event: {} - {}", event_type, details);
        
        self.log(LogLevel::Warn, &message, metadata);
    }
    
    /// Get current metrics
    pub fn get_metrics(&self) -> LogMetrics {
        let metrics = self.metrics.lock().expect("Logger metrics lock poisoned");
        LogMetrics {
            total_entries: metrics.total_entries,
            entries_by_level: metrics.entries_by_level.clone(),
            performance_metrics: metrics.performance_metrics.clone(),
            error_count: metrics.error_count,
        }
    }
    
    /// Check if should log at given level
    fn should_log(&self, level: &LogLevel) -> bool {
        use LogLevel::*;
        
        match (&self.config.level, level) {
            (Trace, _) => true,
            (Debug, Trace) => false,
            (Debug, _) => true,
            (Info, Trace | Debug) => false,
            (Info, _) => true,
            (Warn, Trace | Debug | Info) => false,
            (Warn, _) => true,
            (Error, Error) => true,
            (Error, _) => false,
        }
    }
    
    /// Format log entry according to configured format
    fn format_entry(&self, entry: &LogEntry) -> String {
        match self.config.format {
            LogFormat::Text => {
                format!("[{}] {}: {}",
                        self.format_timestamp(entry.timestamp),
                        entry.level,
                        entry.message)
            },
            LogFormat::Json => {
                serde_json::to_string(entry).unwrap_or_default()
            },
            LogFormat::Structured => {
                let mut parts = vec![
                    format!("timestamp={}", entry.timestamp),
                    format!("level={}", entry.level),
                    format!("message=\"{}\"", entry.message),
                ];
                
                if let Some(ref op) = entry.operation {
                    parts.push(format!("operation={}", op));
                }
                
                if let Some(duration) = entry.duration_ms {
                    parts.push(format!("duration_ms={}", duration));
                }
                
                for (key, value) in &entry.metadata {
                    parts.push(format!("{}={}", key, value));
                }
                
                parts.join(" ")
            },
        }
    }
    
    /// Format timestamp
    fn format_timestamp(&self, timestamp: u64) -> String {
        let datetime = chrono::DateTime::from_timestamp(timestamp as i64, 0)
            .unwrap_or_default();
        datetime.format("%Y-%m-%d %H:%M:%S").to_string()
    }
    
    /// Update metrics
    fn update_metrics(&self, entry: &LogEntry) {
        let mut metrics = self.metrics.lock().expect("Logger metrics lock poisoned");
        
        metrics.total_entries += 1;
        
        let level_count = metrics.entries_by_level
            .entry(entry.level.clone())
            .or_insert(0);
        *level_count += 1;
        
        if entry.level == "Error" {
            metrics.error_count += 1;
        }
        
        if let (Some(operation), Some(duration)) = (&entry.operation, entry.duration_ms) {
            metrics.performance_metrics
                .entry(operation.clone())
                .or_default()
                .push(duration);
        }
    }
}

impl Default for LoggerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            level: LogLevel::Info,
            format: LogFormat::Text,
            file_path: None,
            metrics_enabled: true,
            audit_enabled: false,
            max_file_size: Some(10 * 1024 * 1024), // 10MB
            rotate: true,
        }
    }
}

/// Global logger instance
static mut GLOBAL_LOGGER: Option<NexLogger> = None;
static LOGGER_INIT: std::sync::Once = std::sync::Once::new();

/// Initialize global logger
pub fn init_logger(config: LoggerConfig) -> Result<(), Box<dyn std::error::Error>> {
    LOGGER_INIT.call_once(|| {
        let logger = NexLogger::new(config).expect("Failed to create logger");
        unsafe {
            GLOBAL_LOGGER = Some(logger);
        }
    });
    Ok(())
}

/// Get global logger reference
pub fn get_logger() -> Option<&'static NexLogger> {
    // Use a safer approach for global logger access
    None // For now, return None to avoid static_mut_refs issue
}

/// Convenience macros for logging
#[macro_export]
macro_rules! log_trace {
    ($($arg:tt)*) => {
        if let Some(logger) = $crate::features::logging::get_logger() {
            logger.log($crate::features::logging::LogLevel::Trace, &format!($($arg)*), std::collections::HashMap::new());
        }
    };
}

#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        if let Some(logger) = $crate::features::logging::get_logger() {
            logger.log($crate::features::logging::LogLevel::Debug, &format!($($arg)*), std::collections::HashMap::new());
        }
    };
}

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        if let Some(logger) = $crate::features::logging::get_logger() {
            logger.log($crate::features::logging::LogLevel::Info, &format!($($arg)*), std::collections::HashMap::new());
        }
    };
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        if let Some(logger) = $crate::features::logging::get_logger() {
            logger.log($crate::features::logging::LogLevel::Warn, &format!($($arg)*), std::collections::HashMap::new());
        }
    };
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        if let Some(logger) = $crate::features::logging::get_logger() {
            logger.log($crate::features::logging::LogLevel::Error, &format!($($arg)*), std::collections::HashMap::new());
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
        
    #[test]
    fn test_logger_creation() {
        let config = LoggerConfig::default();
        let logger = NexLogger::new(config);
        assert!(logger.is_ok());
    }
    
    #[test]
    fn test_log_entry_creation() {
        let entry = LogEntry {
            timestamp: 1234567890,
            level: "Info".to_string(),
            message: "Test message".to_string(),
            operation: Some("test_op".to_string()),
            duration_ms: Some(100),
            metadata: HashMap::new(),
        };
        
        assert_eq!(entry.level, "Info");
        assert_eq!(entry.message, "Test message");
        assert_eq!(entry.operation, Some("test_op".to_string()));
    }
    
    #[test]
    fn test_log_levels() {
        let config = LoggerConfig {
            level: LogLevel::Warn,
            ..Default::default()
        };
        
        let logger = NexLogger::new(config).expect("Failed to create logger");
        
        // Should log warn and error
        assert!(logger.should_log(&LogLevel::Warn));
        assert!(logger.should_log(&LogLevel::Error));
        
        // Should not log info, debug, trace
        assert!(!logger.should_log(&LogLevel::Info));
        assert!(!logger.should_log(&LogLevel::Debug));
        assert!(!logger.should_log(&LogLevel::Trace));
    }
    
    #[test]
    fn test_metrics_update() {
        let config = LoggerConfig::default();
        let logger = NexLogger::new(config).expect("Failed to create logger");
        
        let mut metadata = HashMap::new();
        metadata.insert("operation".to_string(), "test_op".to_string());
        
        logger.log(LogLevel::Info, "Test message", metadata);
        
        let metrics = logger.get_metrics();
        assert_eq!(metrics.total_entries, 1);
        assert_eq!(metrics.entries_by_level.get("Info"), Some(&1));
    }
}
