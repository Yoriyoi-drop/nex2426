//! Comprehensive logging and monitoring for NEX2426
//! 
//! Provides structured logging, performance monitoring, and security auditing
//! capabilities for production environments.

use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};
use serde::{Serialize, Deserialize};

/// Log levels for NEX2426
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum LogLevel {
    Trace = 0,
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
    Critical = 5,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Structured log entry
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub level: LogLevel,
    pub module: String,
    pub message: String,
    pub context: Option<serde_json::Value>,
    pub duration_ms: Option<u64>,
    pub thread_id: Option<std::thread::ThreadId>,
}

/// Performance metrics
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub operation: String,
    pub duration_ms: u64,
    pub memory_used_mb: Option<f64>,
    pub cpu_usage_percent: Option<f64>,
    pub success: bool,
    pub error_message: Option<String>,
}

/// Security audit entry
#[derive(Debug, Clone)]
pub struct SecurityAudit {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub event_type: SecurityEventType,
    pub user_id: Option<String>,
    pub resource: String,
    pub action: String,
    pub outcome: SecurityOutcome,
    pub details: Option<serde_json::Value>,
}

/// Security event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityEventType {
    Authentication,
    Authorization,
    KeyGeneration,
    Encryption,
    Decryption,
    DataAccess,
    ConfigurationChange,
    SystemError,
}

/// Security outcomes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityOutcome {
    Success,
    Failure,
    Blocked,
    Suspicious,
}

/// Logger configuration
#[derive(Debug, Clone)]
pub struct LoggerConfig {
    pub level: LogLevel,
    pub enable_file_logging: bool,
    pub enable_console_logging: bool,
    pub log_file_path: Option<String>,
    pub max_file_size_mb: u64,
    pub enable_performance_monitoring: bool,
    pub enable_security_audit: bool,
}

impl Default for LoggerConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::Info,
            enable_file_logging: true,
            enable_console_logging: true,
            log_file_path: Some("nex2426.log".to_string()),
            max_file_size_mb: 100,
            enable_performance_monitoring: true,
            enable_security_audit: true,
        }
    }
}

/// Main logger for NEX2426
pub struct NexLogger {
    config: LoggerConfig,
    performance_metrics: Arc<Mutex<Vec<PerformanceMetrics>>>,
    security_audit: Arc<Mutex<Vec<SecurityAudit>>>,
    start_time: Instant,
}

impl NexLogger {
    /// Create new logger with configuration
    pub fn new(config: LoggerConfig) -> Self {
        Self {
            config,
            performance_metrics: Arc::new(Mutex::new(Vec::new())),
            security_audit: Arc::new(Mutex::new(Vec::new())),
            start_time: Instant::now(),
        }
    }
    
    /// Log a message with context
    pub fn log(&self, level: LogLevel, module: &str, message: &str, context: Option<serde_json::Value>) {
        if level < self.config.level {
            return;
        }
        
        let entry = LogEntry {
            timestamp: chrono::Utc::now(),
            level,
            module: module.to_string(),
            message: message.to_string(),
            context,
            duration_ms: None,
            thread_id: Some(std::thread::current().id()),
        };
        
        // Log to console
        if self.config.enable_console_logging {
            self.log_to_console(&entry);
        }
        
        // Log to file
        if self.config.enable_file_logging {
            if let Err(e) = self.log_to_file(&entry) {
                eprintln!("Failed to write to log file: {}", e);
            }
        }
    }
    
    /// Log performance metrics
    pub fn log_performance(&self, metrics: PerformanceMetrics) {
        if !self.config.enable_performance_monitoring {
            return;
        }
        
        let mut metrics_list = self.performance_metrics.lock().unwrap();
        metrics_list.push(metrics);
        
        // Keep only last 1000 metrics
        if metrics_list.len() > 1000 {
            metrics_list.drain(0..500);
        }
    }
    
    /// Log security audit
    pub fn log_security(&self, audit: SecurityAudit) {
        if !self.config.enable_security_audit {
            return;
        }
        
        let mut audit_list = self.security_audit.lock().unwrap();
        audit_list.push(audit);
        
        // Keep only last 1000 audit entries
        if audit_list.len() > 1000 {
            audit_list.drain(0..500);
        }
    }
    
    /// Get performance statistics
    pub fn get_performance_stats(&self) -> PerformanceStats {
        let metrics = self.performance_metrics.lock().unwrap();
        
        if metrics.is_empty() {
            return PerformanceStats::default();
        }
        
        let total_operations = metrics.len();
        let successful_operations = metrics.iter().filter(|m| m.success).count();
        let total_duration: u64 = metrics.iter().map(|m| m.duration_ms).sum();
        let avg_duration = total_duration / total_operations as u64;
        
        let memory_usage: Vec<f64> = metrics.iter()
            .filter_map(|m| m.memory_used_mb)
            .collect();
        let avg_memory = if memory_usage.is_empty() {
            0.0
        } else {
            memory_usage.iter().sum::<f64>() / memory_usage.len() as f64
        };
        
        PerformanceStats {
            total_operations,
            success_rate: successful_operations as f64 / total_operations as f64,
            avg_duration_ms: avg_duration,
            avg_memory_mb: avg_memory,
            uptime_seconds: self.start_time.elapsed().as_secs(),
        }
    }
    
    /// Get security audit summary
    pub fn get_security_summary(&self) -> SecuritySummary {
        let audit = self.security_audit.lock().unwrap();
        
        if audit.is_empty() {
            return SecuritySummary::default();
        }
        
        let total_events = audit.len();
        let successful_events = audit.iter()
            .filter(|a| matches!(a.outcome, SecurityOutcome::Success))
            .count();
        let blocked_events = audit.iter()
            .filter(|a| matches!(a.outcome, SecurityOutcome::Blocked))
            .count();
        let suspicious_events = audit.iter()
            .filter(|a| matches!(a.outcome, SecurityOutcome::Suspicious))
            .count();
        
        SecuritySummary {
            total_events,
            successful_events,
            blocked_events,
            suspicious_events,
            success_rate: successful_events as f64 / total_events as f64,
        }
    }
    
    /// Export logs to JSON (simplified version)
    pub fn export_logs(&self) -> crate::error::NexResult<String> {
        let metrics = self.performance_metrics.lock().unwrap();
        let uptime = self.start_time.elapsed().as_secs();
        
        // Simple JSON without serialization
        let json_str = format!(
            r#"{{"timestamp": "{}", "uptime_seconds": {}, "total_metrics": {}}}"#,
            chrono::Utc::now().to_rfc3339(),
            uptime,
            metrics.len()
        );
        
        Ok(json_str)
    }
    
    /// Log to console
    fn log_to_console(&self, entry: &LogEntry) {
        let timestamp = entry.timestamp.format("%Y-%m-%d %H:%M:%S%.3f");
        let thread_id = entry.thread_id
            .map(|id| format!("{:?}", id))
            .unwrap_or_else(|| "unknown".to_string());
        
        let context_str = entry.context
            .as_ref()
            .map(|c| format!(" | Context: {}", c))
            .unwrap_or_default();
        
        match entry.level {
            LogLevel::Trace | LogLevel::Debug => {
                println!("[{} {} {} {}]{} {}", 
                         timestamp, entry.level, entry.module, thread_id, entry.message, context_str);
            }
            LogLevel::Info => {
                println!("[{} {} {} {}]{} {}", 
                         timestamp, entry.level, entry.module, thread_id, entry.message, context_str);
            }
            LogLevel::Warn => {
                eprintln!("[{} {} {} {}]{} {}", 
                          timestamp, entry.level, entry.module, thread_id, entry.message, context_str);
            }
            LogLevel::Error | LogLevel::Critical => {
                eprintln!("[{} {} {} {}]{} {}", 
                          timestamp, entry.level, entry.module, thread_id, entry.message, context_str);
            }
        }
    }
    
    /// Log to file
    fn log_to_file(&self, entry: &LogEntry) -> crate::error::NexResult<()> {
        use std::fs::OpenOptions;
        use std::io::Write;
        
        let log_path = self.config.log_file_path
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or("nex2426.log");
        
        // Simple JSON log format without serde
        let json_line = format!(
            r#"{{"timestamp": "{}", "level": "{}", "module": "{}", "message": "{}"}}"#,
            entry.timestamp.to_rfc3339(),
            entry.level,
            entry.module,
            entry.message
        );
        
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)?;
        
        writeln!(file, "{}", json_line)?;
        file.flush()?;
        
        // Check file size and rotate if necessary
        if let Ok(metadata) = file.metadata() {
            if metadata.len() > self.config.max_file_size_mb * 1024 * 1024 {
                self.rotate_log_file(log_path)?;
            }
        }
        
        Ok(())
    }
    
    /// Rotate log file when it gets too large
    fn rotate_log_file(&self, log_path: &str) -> crate::error::NexResult<()> {
        use std::fs;
        
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let backup_path = format!("{}.{}", log_path, timestamp);
        
        fs::rename(log_path, &backup_path)?;
        eprintln!("Log file rotated to: {}", backup_path);
        
        Ok(())
    }
}

/// Performance statistics
#[derive(Debug, Clone)]
pub struct PerformanceStats {
    pub total_operations: usize,
    pub success_rate: f64,
    pub avg_duration_ms: u64,
    pub avg_memory_mb: f64,
    pub uptime_seconds: u64,
}

impl Default for PerformanceStats {
    fn default() -> Self {
        Self {
            total_operations: 0,
            success_rate: 0.0,
            avg_duration_ms: 0,
            avg_memory_mb: 0.0,
            uptime_seconds: 0,
        }
    }
}

/// Security summary
#[derive(Debug, Clone)]
pub struct SecuritySummary {
    pub total_events: usize,
    pub successful_events: usize,
    pub blocked_events: usize,
    pub suspicious_events: usize,
    pub success_rate: f64,
}

impl Default for SecuritySummary {
    fn default() -> Self {
        Self {
            total_events: 0,
            successful_events: 0,
            blocked_events: 0,
            suspicious_events: 0,
            success_rate: 0.0,
        }
    }
}

/// Global logger instance
lazy_static::lazy_static! {
    static ref GLOBAL_LOGGER: Arc<Mutex<Option<NexLogger>>> = Arc::new(Mutex::new(None));
}

/// Initialize global logger
pub fn init_logger(config: LoggerConfig) -> crate::error::NexResult<()> {
    let mut global_logger = GLOBAL_LOGGER.lock().unwrap();
    *global_logger = Some(NexLogger::new(config));
    Ok(())
}

/// Log a message using global logger
pub fn log(level: LogLevel, module: &str, message: &str) {
    if let Some(logger) = GLOBAL_LOGGER.lock().unwrap().as_ref() {
        logger.log(level, module, message, None);
    }
}

/// Log a message with context using global logger
pub fn log_with_context(level: LogLevel, module: &str, message: &str, context: serde_json::Value) {
    if let Some(logger) = GLOBAL_LOGGER.lock().unwrap().as_ref() {
        logger.log(level, module, message, Some(context));
    }
}

/// Log performance metrics using global logger
pub fn log_performance(metrics: PerformanceMetrics) {
    if let Some(logger) = GLOBAL_LOGGER.lock().unwrap().as_ref() {
        logger.log_performance(metrics);
    }
}

/// Log security audit using global logger
pub fn log_security(audit: SecurityAudit) {
    if let Some(logger) = GLOBAL_LOGGER.lock().unwrap().as_ref() {
        logger.log_security(audit);
    }
}

/// Get performance statistics from global logger
pub fn get_performance_stats() -> Option<PerformanceStats> {
    GLOBAL_LOGGER.lock().unwrap()
        .as_ref()
        .map(|logger| logger.get_performance_stats())
}

/// Get security summary from global logger
pub fn get_security_summary() -> Option<SecuritySummary> {
    GLOBAL_LOGGER.lock().unwrap()
        .as_ref()
        .map(|logger| logger.get_security_summary())
}

/// Export logs from global logger
pub fn export_logs() -> crate::error::NexResult<String> {
    if let Some(logger) = GLOBAL_LOGGER.lock().unwrap().as_ref() {
        let metrics = logger.performance_metrics.lock().unwrap();
        
        // Simple JSON export without complex serialization
        let export_json = format!(
            r#"{{"timestamp": "{}", "uptime_seconds": {}, "total_operations": {}}}"#,
            chrono::Utc::now().to_rfc3339(),
            logger.start_time.elapsed().as_secs(),
            metrics.len()
        );
        
        Ok(export_json)
    } else {
        Err(crate::error::NexError::internal("Logger not initialized"))
    }
}

/// Macro for convenient logging
#[macro_export]
macro_rules! nex_log {
    (trace, $module:expr, $msg:expr) => {
        $crate::logging::log($crate::logging::LogLevel::Trace, $module, $msg)
    };
    (debug, $module:expr, $msg:expr) => {
        $crate::logging::log($crate::logging::LogLevel::Debug, $module, $msg)
    };
    (info, $module:expr, $msg:expr) => {
        $crate::logging::log($crate::logging::LogLevel::Info, $module, $msg)
    };
    (warn, $module:expr, $msg:expr) => {
        $crate::logging::log($crate::logging::LogLevel::Warn, $module, $msg)
    };
    (error, $module:expr, $msg:expr) => {
        $crate::logging::log($crate::logging::LogLevel::Error, $module, $msg)
    };
    (critical, $module:expr, $msg:expr) => {
        $crate::logging::log($crate::logging::LogLevel::Critical, $module, $msg)
    };
    (trace, $module:expr, $msg:expr, $context:expr) => {
        $crate::logging::log_with_context($crate::logging::LogLevel::Trace, $module, $msg, $context)
    };
    (debug, $module:expr, $msg:expr, $context:expr) => {
        $crate::logging::log_with_context($crate::logging::LogLevel::Debug, $module, $msg, $context)
    };
    (info, $module:expr, $msg:expr, $context:expr) => {
        $crate::logging::log_with_context($crate::logging::LogLevel::Info, $module, $msg, $context)
    };
    (warn, $module:expr, $msg:expr, $context:expr) => {
        $crate::logging::log_with_context($crate::logging::LogLevel::Warn, $module, $msg, $context)
    };
    (error, $module:expr, $msg:expr, $context:expr) => {
        $crate::logging::log_with_context($crate::logging::LogLevel::Error, $module, $msg, $context)
    };
    (critical, $module:expr, $msg:expr, $context:expr) => {
        $crate::logging::log_with_context($crate::logging::LogLevel::Critical, $module, $msg, $context)
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_levels() {
        assert!(LogLevel::Info > LogLevel::Debug);
        assert!(LogLevel::Error < LogLevel::Critical);
    }

    #[test]
    fn test_performance_metrics() {
        let metrics = PerformanceMetrics {
            operation: "test".to_string(),
            duration_ms: 100,
            memory_used_mb: Some(50.0),
            cpu_usage_percent: None,
            success: true,
            error_message: None,
        };
        
        assert_eq!(metrics.operation, "test");
        assert_eq!(metrics.duration_ms, 100);
    }

    #[test]
    fn test_security_audit() {
        let audit = SecurityAudit {
            timestamp: chrono::Utc::now(),
            event_type: SecurityEventType::Authentication,
            user_id: Some("user123".to_string()),
            resource: "system".to_string(),
            action: "login".to_string(),
            outcome: SecurityOutcome::Success,
            details: None,
        };
        
        assert!(matches!(audit.event_type, SecurityEventType::Authentication));
        assert!(matches!(audit.outcome, SecurityOutcome::Success));
    }
}
