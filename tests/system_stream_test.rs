use anyhow::Result;
use serde_json::json;
use std::time::Duration;
use tokio::time::timeout;
use tracing::{info, Level};
use std::collections::HashMap;

use unison_protocol::network::{
    Service, ServiceConfig, ServicePriority, UnisonService,
    SystemStream, UnisonStream, StreamHandle, NetworkError
};

/// Test SystemStream basic functionality
#[tokio::test]
async fn test_system_stream_basic() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_test_writer()
        .init();

    info!("🧪 Testing SystemStream basic functionality");

    // Test StreamHandle creation
    let handle = StreamHandle {
        stream_id: 1,
        method: "test_method".to_string(),
        created_at: std::time::SystemTime::now(),
    };

    info!("✅ StreamHandle created: {:?}", handle);
    assert_eq!(handle.stream_id, 1);
    assert_eq!(handle.method, "test_method");

    Ok(())
}

/// Test Service configuration and metadata
#[tokio::test]
async fn test_service_config() -> Result<()> {
    info!("🧪 Testing Service configuration");

    // Test default service configuration
    let default_config = ServiceConfig::default();
    assert_eq!(default_config.service_name, "unison-service");
    assert_eq!(default_config.service_version, "1.0.0");
    assert_eq!(default_config.priority, ServicePriority::Normal);
    assert!(default_config.reliable_delivery);
    assert!(default_config.ordered_delivery);

    info!("✅ Default ServiceConfig validated");

    // Test custom service configuration
    let custom_config = ServiceConfig {
        service_name: "test-service".to_string(),
        service_version: "2.0.0".to_string(),
        priority: ServicePriority::High,
        max_concurrent_requests: 50,
        request_timeout: Duration::from_secs(15),
        ..Default::default()
    };

    assert_eq!(custom_config.service_name, "test-service");
    assert_eq!(custom_config.service_version, "2.0.0");
    assert_eq!(custom_config.priority, ServicePriority::High);
    assert_eq!(custom_config.max_concurrent_requests, 50);

    info!("✅ Custom ServiceConfig validated");

    Ok(())
}

/// Test ServicePriority ordering
#[tokio::test]
async fn test_service_priority() -> Result<()> {
    info!("🧪 Testing ServicePriority ordering");

    assert!(ServicePriority::Critical as u8 > ServicePriority::High as u8);
    assert!(ServicePriority::High as u8 > ServicePriority::Normal as u8);
    assert!(ServicePriority::Normal as u8 > ServicePriority::Low as u8);

    info!("✅ ServicePriority ordering validated");

    Ok(())
}

/// Mock SystemStream for testing
pub struct MockSystemStream {
    stream_id: u64,
    method: String,
    active: bool,
    received_messages: Vec<serde_json::Value>,
    sent_messages: Vec<serde_json::Value>,
}

impl MockSystemStream {
    pub fn new(stream_id: u64, method: String) -> Self {
        Self {
            stream_id,
            method,
            active: true,
            received_messages: Vec::new(),
            sent_messages: Vec::new(),
        }
    }
}

#[async_trait::async_trait]
impl SystemStream for MockSystemStream {
    async fn send(&mut self, data: serde_json::Value) -> Result<(), NetworkError> {
        if !self.active {
            return Err(NetworkError::Connection("Stream is not active".to_string()));
        }
        self.sent_messages.push(data);
        Ok(())
    }
    
    async fn receive(&mut self) -> Result<serde_json::Value, NetworkError> {
        if !self.active {
            return Err(NetworkError::Connection("Stream is not active".to_string()));
        }
        
        // Return a mock response
        Ok(json!({
            "type": "mock_response",
            "stream_id": self.stream_id,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
    }
    
    fn is_active(&self) -> bool {
        self.active
    }
    
    async fn close(&mut self) -> Result<(), NetworkError> {
        self.active = false;
        Ok(())
    }
    
    fn get_handle(&self) -> StreamHandle {
        StreamHandle {
            stream_id: self.stream_id,
            method: self.method.clone(),
            created_at: std::time::SystemTime::now(),
        }
    }
}

/// Test UnisonService with mock SystemStream
#[tokio::test]
async fn test_unison_service_with_mock() -> Result<()> {
    info!("🧪 Testing UnisonService with MockSystemStream");

    let config = ServiceConfig {
        service_name: "test-service".to_string(),
        service_version: "1.0.0".to_string(),
        ..Default::default()
    };

    let mock_stream = MockSystemStream::new(1, "test_method".to_string());
    let boxed_stream: Box<dyn SystemStream> = Box::new(mock_stream);
    
    let mut service = UnisonService::new(config, boxed_stream);

    // Test service metadata
    let metadata = service.metadata();
    assert_eq!(metadata.get("service_name").unwrap(), "test-service");
    assert_eq!(metadata.get("service_version").unwrap(), "1.0.0");

    info!("✅ Service metadata: {:?}", metadata);

    // Test service basic functionality
    assert_eq!(service.service_name(), "test-service");
    assert_eq!(service.version(), "1.0.0");
    assert_eq!(service.service_type(), "unison-service");

    info!("✅ Service basic functionality validated");

    // Test service capabilities
    let capabilities = service.get_capabilities();
    assert!(capabilities.contains(&"ping".to_string()));
    assert!(capabilities.contains(&"get_stats".to_string()));

    info!("✅ Service capabilities: {:?}", capabilities);

    // Test handle_request
    let ping_response = service.handle_request("ping", json!({})).await?;
    assert!(ping_response.get("service").is_some());
    assert!(ping_response.get("status").is_some());
    
    info!("✅ Ping response: {}", ping_response);

    // Test get_stats request
    let stats_response = service.handle_request("get_stats", json!({})).await?;
    assert!(stats_response.get("requests_processed").is_some());
    
    info!("✅ Stats response: {}", stats_response);

    // Test invalid method
    let result = service.handle_request("invalid_method", json!({})).await;
    assert!(result.is_err());
    
    info!("✅ Invalid method properly rejected");

    Ok(())
}

/// Test service metadata and structured communication
#[tokio::test]
async fn test_service_metadata_communication() -> Result<()> {
    info!("🧪 Testing service metadata communication");

    let config = ServiceConfig::default();
    let mock_stream = MockSystemStream::new(2, "metadata_test".to_string());
    let boxed_stream: Box<dyn SystemStream> = Box::new(mock_stream);
    
    let mut service = UnisonService::new(config, boxed_stream);

    // Test send_with_metadata
    let test_data = json!({"message": "Hello, World!"});
    let metadata = HashMap::from([
        ("priority".to_string(), "high".to_string()),
        ("source".to_string(), "test".to_string()),
    ]);

    service.send_with_metadata(test_data, metadata).await?;
    
    info!("✅ Metadata communication sent successfully");

    // Test service ping
    service.service_ping().await?;
    
    info!("✅ Service ping sent successfully");

    // Test service heartbeat start
    service.start_service_heartbeat(30).await?;
    
    info!("✅ Service heartbeat started successfully");

    Ok(())
}

/// Test SystemStream handle functionality
#[tokio::test]
async fn test_stream_handle() -> Result<()> {
    info!("🧪 Testing StreamHandle functionality");

    let handle = StreamHandle {
        stream_id: 12345,
        method: "test_stream_method".to_string(),
        created_at: std::time::SystemTime::now(),
    };

    // Test handle serialization
    let serialized = serde_json::to_string(&handle)?;
    let deserialized: StreamHandle = serde_json::from_str(&serialized)?;

    assert_eq!(handle.stream_id, deserialized.stream_id);
    assert_eq!(handle.method, deserialized.method);
    
    info!("✅ StreamHandle serialization/deserialization works");
    info!("   Stream ID: {}", handle.stream_id);
    info!("   Method: {}", handle.method);

    Ok(())
}

/// Performance test for service operations
#[tokio::test]
async fn test_service_performance() -> Result<()> {
    info!("🧪 Testing service performance");

    let config = ServiceConfig::default();
    let mock_stream = MockSystemStream::new(3, "performance_test".to_string());
    let boxed_stream: Box<dyn SystemStream> = Box::new(mock_stream);
    
    let mut service = UnisonService::new(config, boxed_stream);

    let start_time = std::time::Instant::now();
    
    // Perform 100 ping requests
    for i in 0..100 {
        let _response = service.handle_request("ping", json!({"sequence": i})).await?;
    }
    
    let elapsed = start_time.elapsed();
    let requests_per_sec = 100.0 / elapsed.as_secs_f64();
    
    info!("✅ Performance test completed:");
    info!("   Total time: {:?}", elapsed);
    info!("   Requests per second: {:.2}", requests_per_sec);
    
    // Verify stats were updated
    let stats = service.get_stats();
    assert_eq!(stats.requests_processed, 100);
    
    info!("   Requests processed: {}", stats.requests_processed);

    Ok(())
}