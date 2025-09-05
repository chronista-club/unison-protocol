use anyhow::{Context, Result};
use async_trait::async_trait;
use futures_util::{Stream, StreamExt};
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::{ProtocolClientTrait, ProtocolMessage, MessageType, UnisonClient, NetworkError};

/// Generic protocol client implementation
pub struct ProtocolClient {
    transport: Arc<dyn Transport + Send + Sync>,
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
        Self { transport }
    }
    
    /// Create a new client with default (dummy) transport
    pub fn new_default() -> Self {
        Self { 
            transport: Arc::new(DummyTransport::new())
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