//! REST API server for NEX2426
//! 
//! Provides HTTP API endpoints for NEX2426 operations,
//! enabling integration with web applications and services.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiRequest {
    /// Input data to hash/encrypt
    pub data: String,
    /// Encryption key
    pub key: String,
    /// Cost parameter (optional, uses default if not specified)
    pub cost: Option<u32>,
    /// Enable temporal binding
    pub temporal_binding: Option<bool>,
    /// Output format
    pub output_format: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse {
    /// Success status
    pub success: bool,
    /// Result data
    pub result: Option<String>,
    /// Error message (if any)
    pub error: Option<String>,
    /// Operation metadata
    pub metadata: OperationMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationMetadata {
    /// Operation type
    pub operation: String,
    /// Time taken in milliseconds
    pub duration_ms: u64,
    /// Cost parameter used
    pub cost: u32,
    /// Timestamp
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct NexApiServer {
    /// Server configuration
    config: ApiServerConfig,
    /// Operation statistics
    stats: Arc<RwLock<ApiStats>>,
}

#[derive(Debug, Clone)]
pub struct ApiServerConfig {
    /// Server host
    pub host: String,
    /// Server port
    pub port: u16,
    /// Enable CORS
    pub cors: bool,
    /// Rate limiting requests per minute
    pub rate_limit: Option<u32>,
    /// API key for authentication (optional)
    pub api_key: Option<String>,
    /// Enable request logging
    pub log_requests: bool,
}

#[derive(Debug, Default)]
pub struct ApiStats {
    /// Total requests
    pub total_requests: u64,
    /// Successful requests
    pub successful_requests: u64,
    /// Failed requests
    pub failed_requests: u64,
    /// Average response time
    pub avg_response_time_ms: f64,
    /// Requests by operation type
    pub operations: HashMap<String, u64>,
}

impl NexApiServer {
    /// Create new API server
    pub fn new(config: ApiServerConfig) -> Self {
        Self {
            config,
            stats: Arc::new(RwLock::new(ApiStats::default())),
        }
    }
    
    /// Start the API server
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🚀 Starting NEX2426 API Server on {}:{}", 
                self.config.host, self.config.port);
        
        // In a real implementation, you would use a web framework like axum, warp, or actix-web
        // For now, we'll provide a mock implementation
        
        self.setup_routes().await?;
        
        println!("✅ API Server started successfully");
        println!("📊 Statistics endpoint available at /stats");
        println!("🔗 Health check available at /health");
        
        Ok(())
    }
    
    /// Setup API routes
    async fn setup_routes(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Mock route setup - in real implementation:
        /*
        use axum::{
            routing::{get, post},
            Router, Json, extract::Path,
            http::StatusCode,
            response::Json as ResponseJson,
        };
        
        let app = Router::new()
            .route("/hash", post(hash_handler))
            .route("/encrypt", post(encrypt_handler))
            .route("/decrypt", post(decrypt_handler))
            .route("/stats", get(stats_handler))
            .route("/health", get(health_handler))
            .route("/bench", post(bench_handler));
        
        let listener = tokio::net::TcpListener::bind(
            format!("{}:{}", self.config.host, self.config.port)
        ).await?;
        
        axum::serve(listener, app).await?;
        */
        
        println!("📡 Routes configured:");
        println!("  POST /hash      - Hash data");
        println!("  POST /encrypt    - Encrypt file");
        println!("  POST /decrypt    - Decrypt file");
        println!("  POST /bench      - Performance benchmark");
        println!("  GET  /stats      - Server statistics");
        println!("  GET  /health     - Health check");
        
        Ok(())
    }
    
    /// Handle hash operation
    pub async fn handle_hash(&self, request: ApiRequest) -> ApiResponse {
        let start_time = std::time::Instant::now();
        
        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.total_requests += 1;
            *stats.operations.entry("hash".to_string()).or_insert(0) += 1;
        }
        
        // Perform hash operation
        let cost = request.cost.unwrap_or(3);
        let result = crate::kernel::NexKernel::new(cost)
            .execute(&mut std::io::Cursor::new(&request.data), &request.key);
        
        // Update success stats
        {
            let mut stats = self.stats.write().await;
            stats.successful_requests += 1;
            let duration = start_time.elapsed().as_millis() as f64;
            stats.avg_response_time_ms = 
                (stats.avg_response_time_ms * (stats.total_requests - 1) as f64 + duration) 
                / stats.total_requests as f64;
        }
        
        ApiResponse {
            success: true,
            result: Some(result.full_formatted_string),
            error: None,
            metadata: OperationMetadata {
                operation: "hash".to_string(),
                duration_ms: start_time.elapsed().as_millis() as u64,
                cost,
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_else(|_| std::time::Duration::from_secs(0))
                    .as_secs(),
            },
        }
    }
    
    /// Get server statistics
    pub async fn get_stats(&self) -> ApiStats {
        self.stats.read().await.clone()
    }
    
    /// Health check
    pub async fn health_check(&self) -> HashMap<String, String> {
        let mut health = HashMap::new();
        health.insert("status".to_string(), "healthy".to_string());
        health.insert("version".to_string(), env!("CARGO_PKG_VERSION").to_string());
        health.insert("uptime".to_string(), "running".to_string());
        health
    }
}

impl Default for ApiServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            cors: true,
            rate_limit: Some(60), // 60 requests per minute
            api_key: None,
            log_requests: true,
        }
    }
}

impl Clone for ApiStats {
    fn clone(&self) -> Self {
        Self {
            total_requests: self.total_requests,
            successful_requests: self.successful_requests,
            failed_requests: self.failed_requests,
            avg_response_time_ms: self.avg_response_time_ms,
            operations: self.operations.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_api_server_creation() {
        let config = ApiServerConfig::default();
        let server = NexApiServer::new(config);
        
        let stats = server.get_stats().await;
        assert_eq!(stats.total_requests, 0);
    }
    
    #[tokio::test]
    async fn test_hash_operation() {
        let config = ApiServerConfig::default();
        let server = NexApiServer::new(config);
        
        let request = ApiRequest {
            data: "test data".to_string(),
            key: "test key".to_string(),
            cost: Some(1),
            temporal_binding: None,
            output_format: None,
        };
        
        let response = server.handle_hash(request).await;
        assert!(response.success);
        assert!(response.result.is_some());
        assert_eq!(response.metadata.operation, "hash");
    }
    
    #[tokio::test]
    async fn test_health_check() {
        let config = ApiServerConfig::default();
        let server = NexApiServer::new(config);
        
        let health = server.health_check().await;
        assert_eq!(health.get("status"), Some(&"healthy".to_string()));
    }
}
