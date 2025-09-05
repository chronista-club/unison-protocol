use anyhow::{Context, Result};
use async_trait::async_trait;
use futures_util::{Stream, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::{ProtocolClientTrait, ProtocolMessage, MessageType, UnisonClient, NetworkError, SystemStream, UnisonClientExt};
use super::service::{Service, UnisonService, ServiceConfig};

/// Generic protocol client implementation
pub struct ProtocolClient {
    transport: Arc<dyn Transport + Send + Sync>,
    services: Arc<RwLock<HashMap<String, Box<dyn Service>>>>,
}

/// Transport trait for underlying communication
#[async_trait]
pub trait Transport: Send + Sync {
    async fn send(&self, message: ProtocolMessage) -> Result<()>;
    async fn receive(&self) -> Result<ProtocolMessage>;
    async fn connect(&self, url: &str) -> Result<()>;
    async fn disconnect(&self) -> Result<()>;
    async fn is_connected(&self) -> bool;
}

impl ProtocolClient {
    pub fn new(transport: Arc<dyn Transport + Send + Sync>) -> Self {
        Self { 
            transport,
            services: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Create a new client with default QUIC transport
    pub fn new_quic() -> Result<Self> {
        use super::quic::QuicClient;
        let transport = Arc::new(QuicClient::new()?);
        Ok(Self { 
            transport,
            services: Arc::new(RwLock::new(HashMap::new())),
        })
    }
    
    /// Create a new client with default (dummy) transport
    pub fn new_default() -> Self {
        Self { 
            transport: Arc::new(DummyTransport::new()),
            services: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Register a Service instance with the client
    pub async fn register_service(&self, service: Box<dyn Service>) {
        let service_name = service.service_name().to_string();
        let mut services = self.services.write().await;
        services.insert(service_name, service);
    }
    
    /// Get registered services list
    pub async fn list_services(&self) -> Vec<String> {
        let services = self.services.read().await;
        services.keys().cloned().collect()
    }
    
    /// Call a service method directly
    pub async fn call_service(&self, service_name: &str, method: &str, payload: serde_json::Value) -> Result<serde_json::Value, NetworkError> {
        let mut services = self.services.write().await;
        if let Some(service) = services.get_mut(service_name) {
            service.handle_request(method, payload).await
        } else {
            Err(NetworkError::HandlerNotFound { method: format!("{}::{}", service_name, method) })
        }
    }
    
    pub async fn connect(&self, url: &str) -> Result<()> {
        self.transport.connect(url).await
    }
    
    pub async fn disconnect(&self) -> Result<()> {
        self.transport.disconnect().await
    }
    
    pub async fn is_connected(&self) -> bool {
        self.transport.is_connected().await
    }
}

#[async_trait]
impl ProtocolClientTrait for ProtocolClient {
    async fn call<TRequest, TResponse>(&self, method: &str, request: TRequest) -> Result<TResponse>
    where
        TRequest: Serialize + Send + Sync,
        TResponse: for<'de> Deserialize<'de>,
    {
        // Generate a unique request ID
        let request_id = generate_request_id();
        
        // Create the protocol message
        let message = ProtocolMessage {
            id: request_id,
            method: method.to_string(),
            msg_type: MessageType::Request,
            payload: serde_json::to_value(request)?,
        };
        
        // Send the request
        self.transport.send(message).await?;
        
        // Wait for the response
        // In a real implementation, this would use a proper request/response correlation mechanism
        let response = self.transport.receive().await?;
        
        if response.msg_type == MessageType::Error {
            return Err(anyhow::anyhow!(
                "Protocol error: {}",
                response.payload.get("message").and_then(|v| v.as_str()).unwrap_or("Unknown error")
            ));
        }
        
        // Deserialize the response
        let result: TResponse = serde_json::from_value(response.payload)
            .context("Failed to deserialize response")?;
        
        Ok(result)
    }
    
    async fn stream<TRequest, TResponse>(
        &self,
        method: &str,
        request: TRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<TResponse>> + Send>>>
    where
        TRequest: Serialize + Send + Sync,
        TResponse: for<'de> Deserialize<'de> + Send + 'static,
    {
        // Generate a unique request ID
        let request_id = generate_request_id();
        
        // Create the protocol message
        let message = ProtocolMessage {
            id: request_id,
            method: method.to_string(),
            msg_type: MessageType::Stream,
            payload: serde_json::to_value(request)?,
        };
        
        // Send the stream request
        self.transport.send(message).await?;
        
        // Create a stream that receives messages
        let transport = Arc::clone(&self.transport);
        let stream = async_stream::stream! {
            loop {
                match transport.receive().await {
                    Ok(msg) => {
                        match msg.msg_type {
                            MessageType::StreamData => {
                                match serde_json::from_value::<TResponse>(msg.payload) {
                                    Ok(data) => yield Ok(data),
                                    Err(e) => yield Err(anyhow::anyhow!("Deserialization error: {}", e)),
                                }
                            }
                            MessageType::StreamEnd => {
                                break;
                            }
                            MessageType::Error => {
                                let error_msg = msg.payload.get("message")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("Unknown error");
                                yield Err(anyhow::anyhow!("Stream error: {}", error_msg));
                                break;
                            }
                            _ => {}
                        }
                    }
                    Err(e) => {
                        yield Err(e);
                        break;
                    }
                }
            }
        };
        
        Ok(Box::pin(stream))
    }
}

fn generate_request_id() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    COUNTER.fetch_add(1, Ordering::SeqCst)
}

#[async_trait]
impl UnisonClient for ProtocolClient {
    async fn connect(&mut self, url: &str) -> Result<(), NetworkError> {
        self.transport.connect(url).await
            .map_err(|e| NetworkError::Connection(e.to_string()))
    }
    
    async fn call(&mut self, method: &str, payload: serde_json::Value) -> Result<serde_json::Value, NetworkError> {
        let request_id = generate_request_id();
        
        let message = ProtocolMessage {
            id: request_id,
            method: method.to_string(),
            msg_type: MessageType::Request,
            payload,
        };
        
        self.transport.send(message).await
            .map_err(|e| NetworkError::Protocol(e.to_string()))?;
        
        let response = self.transport.receive().await
            .map_err(|e| NetworkError::Protocol(e.to_string()))?;
        
        if response.msg_type == MessageType::Error {
            return Err(NetworkError::Protocol(
                response.payload.get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown error")
                    .to_string()
            ));
        }
        
        Ok(response.payload)
    }
    
    async fn disconnect(&mut self) -> Result<(), NetworkError> {
        self.transport.disconnect().await
            .map_err(|e| NetworkError::Connection(e.to_string()))
    }
    
    fn is_connected(&self) -> bool {
        // This would need to be synchronous in the trait
        false // Simplified for now
    }
}

/// Dummy transport for testing/development
struct DummyTransport {
    connected: Arc<RwLock<bool>>,
}

impl DummyTransport {
    fn new() -> Self {
        Self {
            connected: Arc::new(RwLock::new(false)),
        }
    }
}

#[async_trait]
impl Transport for DummyTransport {
    async fn send(&self, _message: ProtocolMessage) -> Result<()> {
        // Dummy implementation
        Ok(())
    }
    
    async fn receive(&self) -> Result<ProtocolMessage> {
        // Dummy implementation - returns a ping response
        Ok(ProtocolMessage {
            id: 1,
            method: "pong".to_string(),
            msg_type: MessageType::Response,
            payload: serde_json::json!({"message": "pong"}),
        })
    }
    
    async fn connect(&self, _url: &str) -> Result<()> {
        let mut connected = self.connected.write().await;
        *connected = true;
        Ok(())
    }
    
    async fn disconnect(&self) -> Result<()> {
        let mut connected = self.connected.write().await;
        *connected = false;
        Ok(())
    }
    
    async fn is_connected(&self) -> bool {
        *self.connected.read().await
    }
}

#[async_trait]
impl UnisonClientExt for ProtocolClient {
    async fn start_system_stream(&mut self, method: &str, payload: serde_json::Value) -> Result<Box<dyn SystemStream>, NetworkError> {
        use super::quic::UnisonStream;
        use super::StreamHandle;
        
        // For now, create a mock SystemStream
        // In a full implementation, this would create a real QUIC bidirectional stream
        let handle = StreamHandle {
            stream_id: generate_request_id(),
            method: method.to_string(),
            created_at: std::time::SystemTime::now(),
        };
        
        // Create a UnisonStream instance
        // Note: This is a placeholder implementation
        // In practice, you would get the actual QUIC connection from the transport
        let stream = Box::new(MockSystemStream::new(handle)) as Box<dyn SystemStream>;
        
        tracing::info!("🌊 Started SystemStream for method: {}", method);
        Ok(stream)
    }
    
    async fn list_system_streams(&self) -> Result<Vec<super::StreamHandle>, NetworkError> {
        // In a full implementation, this would track active streams
        Ok(vec![])
    }
    
    async fn close_system_stream(&mut self, stream_id: u64) -> Result<(), NetworkError> {
        tracing::info!("🔒 Closed SystemStream with ID: {}", stream_id);
        Ok(())
    }
}

/// Mock SystemStream for testing
struct MockSystemStream {
    handle: super::StreamHandle,
    is_active: bool,
}

impl MockSystemStream {
    fn new(handle: super::StreamHandle) -> Self {
        Self {
            handle,
            is_active: true,
        }
    }
}

#[async_trait]
impl SystemStream for MockSystemStream {
    async fn send(&mut self, data: serde_json::Value) -> Result<(), NetworkError> {
        tracing::info!("📤 MockSystemStream send: {}", data);
        Ok(())
    }
    
    async fn receive(&mut self) -> Result<serde_json::Value, NetworkError> {
        tracing::info!("📥 MockSystemStream receive");
        Ok(serde_json::json!({"mock": "response"}))
    }
    
    fn is_active(&self) -> bool {
        self.is_active
    }
    
    async fn close(&mut self) -> Result<(), NetworkError> {
        self.is_active = false;
        tracing::info!("🔒 MockSystemStream closed");
        Ok(())
    }
    
    fn get_handle(&self) -> super::StreamHandle {
        self.handle.clone()
    }
}