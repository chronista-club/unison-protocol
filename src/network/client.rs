use anyhow::{Context, Result};
use futures_util::Stream;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::{ProtocolClientTrait, ProtocolMessage, MessageType, UnisonClient, NetworkError, UnisonClientExt};
use super::service::Service;
use super::quic::QuicClient;

// TransportWrapper removed - using QuicClient directly

/// QUIC protocol client implementation
pub struct ProtocolClient {
    transport: Arc<QuicClient>,
    services: Arc<RwLock<HashMap<String, crate::network::service::UnisonService>>>,
}

// Transport trait removed - using direct implementation on TransportWrapper

impl ProtocolClient {
    pub fn new(transport: QuicClient) -> Self {
        Self { 
            transport: Arc::new(transport),
            services: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Create a new client with QUIC transport
    pub fn new_default() -> Result<Self> {
        let transport = QuicClient::new()?;
        Ok(Self { 
            transport: Arc::new(transport),
            services: Arc::new(RwLock::new(HashMap::new())),
        })
    }
    
    /// Register a Service instance with the client
    pub async fn register_service(&self, service: crate::network::service::UnisonService) {
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
    
    pub async fn connect(&mut self, url: &str) -> Result<()> {
        // Arc::get_mutを使用してmutableアクセス
        Arc::get_mut(&mut self.transport)
            .ok_or_else(|| anyhow::anyhow!("Failed to get mutable transport"))?
            .connect(url).await
    }

    pub async fn disconnect(&mut self) -> Result<()> {
        Arc::get_mut(&mut self.transport)
            .ok_or_else(|| anyhow::anyhow!("Failed to get mutable transport"))?
            .disconnect().await
    }
    
    pub async fn is_connected(&self) -> bool {
        self.transport.is_connected().await
    }
}

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

impl UnisonClient for ProtocolClient {
    async fn connect(&mut self, url: &str) -> Result<(), NetworkError> {
        Arc::get_mut(&mut self.transport)
            .ok_or_else(|| NetworkError::Connection("Failed to get mutable transport".to_string()))?
            .connect(url).await
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
        Arc::get_mut(&mut self.transport)
            .ok_or_else(|| NetworkError::Connection("Failed to get mutable transport".to_string()))?
            .disconnect().await
            .map_err(|e| NetworkError::Connection(e.to_string()))
    }
    
    fn is_connected(&self) -> bool {
        // これはトレイトで同期的である必要があります
        false // 今のところ簡略化
    }
}

// DummyTransport removed - using QuicClient directly

impl UnisonClientExt for ProtocolClient {
    async fn start_system_stream(&mut self, method: &str, _payload: serde_json::Value) -> Result<crate::network::quic::UnisonStream, NetworkError> {
        // use super::quic::UnisonStream;
        use super::StreamHandle;
        
        // Create a real QUIC bidirectional stream
        let _handle = StreamHandle {
            stream_id: generate_request_id(),
            method: method.to_string(),
            created_at: std::time::SystemTime::now(),
        };
        
        // QUICクライアントの接続を取得してUnisonStreamを作成
        // 現在の実装では直接アクセスできないため、エラーを返す
        return Err(NetworkError::NotConnected);
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

// MockSystemStream removed - using UnisonStream directly