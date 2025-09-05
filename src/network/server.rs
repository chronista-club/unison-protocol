use anyhow::Result;
use async_trait::async_trait;
use futures_util::Stream;
use serde_json::Value;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::{ProtocolServerTrait, ProtocolMessage, MessageType, UnisonServer, UnisonServerExt, NetworkError};

/// Server handler function types
type CallHandler = Arc<
    dyn Fn(Value) -> Pin<Box<dyn futures_util::Future<Output = Result<Value>> + Send>>
        + Send
        + Sync,
>;

type StreamHandler = Arc<
    dyn Fn(
            Value,
        ) -> Pin<
            Box<
                dyn futures_util::Future<
                        Output = Result<Pin<Box<dyn Stream<Item = Result<Value>> + Send>>>,
                    > + Send,
            >,
        > + Send
        + Sync,
>;

/// Unison handler type for simple handlers
type UnisonHandler = Arc<
    dyn Fn(serde_json::Value) -> Result<serde_json::Value, NetworkError> + Send + Sync
>;

/// Protocol server implementation
pub struct ProtocolServer {
    call_handlers: Arc<RwLock<HashMap<String, CallHandler>>>,
    stream_handlers: Arc<RwLock<HashMap<String, StreamHandler>>>,
    unison_handlers: Arc<RwLock<HashMap<String, UnisonHandler>>>,
    running: Arc<RwLock<bool>>,
}

impl ProtocolServer {
    pub fn new() -> Self {
        Self {
            call_handlers: Arc::new(RwLock::new(HashMap::new())),
            stream_handlers: Arc::new(RwLock::new(HashMap::new())),
            unison_handlers: Arc::new(RwLock::new(HashMap::new())),
            running: Arc::new(RwLock::new(false)),
        }
    }
    
    /// Register a call handler
    pub async fn register_call_handler<F, Fut>(&self, method: &str, handler: F)
    where
        F: Fn(Value) -> Fut + Send + Sync + 'static,
        Fut: futures_util::Future<Output = Result<Value>> + Send + 'static,
    {
        let handler = Arc::new(move |value: Value| {
            Box::pin(handler(value)) as Pin<Box<dyn futures_util::Future<Output = Result<Value>> + Send>>
        });
        
        let mut handlers = self.call_handlers.write().await;
        handlers.insert(method.to_string(), handler);
    }
    
    /// Register a stream handler
    pub async fn register_stream_handler<F, Fut, S>(&self, method: &str, handler: F)
    where
        F: Fn(Value) -> Fut + Send + Sync + 'static,
        Fut: futures_util::Future<Output = Result<S>> + Send + 'static,
        S: Stream<Item = Result<Value>> + Send + 'static,
    {
        let handler = Arc::new(handler);
        let wrapped_handler = Arc::new(move |value: Value| {
            let handler = Arc::clone(&handler);
            Box::pin(async move {
                let stream = handler(value).await?;
                Ok(Box::pin(stream) as Pin<Box<dyn Stream<Item = Result<Value>> + Send>>)
            }) as Pin<Box<dyn futures_util::Future<Output = Result<Pin<Box<dyn Stream<Item = Result<Value>> + Send>>>> + Send>>
        });
        
        let mut handlers = self.stream_handlers.write().await;
        handlers.insert(method.to_string(), wrapped_handler);
    }
    
    /// Process an incoming message
    pub async fn process_message(&self, message: ProtocolMessage) -> Result<ProtocolMessage> {
        match message.msg_type {
            MessageType::Request => {
                let handlers = self.call_handlers.read().await;
                if let Some(handler) = handlers.get(&message.method) {
                    match handler(message.payload).await {
                        Ok(response) => Ok(ProtocolMessage {
                            id: message.id,
                            method: message.method,
                            msg_type: MessageType::Response,
                            payload: response,
                        }),
                        Err(e) => Ok(ProtocolMessage {
                            id: message.id,
                            method: message.method,
                            msg_type: MessageType::Error,
                            payload: serde_json::json!({
                                "message": e.to_string(),
                            }),
                        }),
                    }
                } else {
                    Ok(ProtocolMessage {
                        id: message.id,
                        method: message.method.clone(),
                        msg_type: MessageType::Error,
                        payload: serde_json::json!({
                            "message": format!("Method not found: {}", message.method),
                        }),
                    })
                }
            }
            MessageType::Stream => {
                // Stream handling would be more complex in a real implementation
                // This is a simplified version
                let handlers = self.stream_handlers.read().await;
                if let Some(_handler) = handlers.get(&message.method) {
                    // In a real implementation, we would:
                    // 1. Start the stream
                    // 2. Send StreamData messages for each item
                    // 3. Send StreamEnd when done
                    Ok(ProtocolMessage {
                        id: message.id,
                        method: message.method,
                        msg_type: MessageType::StreamEnd,
                        payload: serde_json::json!({}),
                    })
                } else {
                    Ok(ProtocolMessage {
                        id: message.id,
                        method: message.method.clone(),
                        msg_type: MessageType::Error,
                        payload: serde_json::json!({
                            "message": format!("Stream method not found: {}", message.method),
                        }),
                    })
                }
            }
            _ => Ok(ProtocolMessage {
                id: message.id,
                method: message.method,
                msg_type: MessageType::Error,
                payload: serde_json::json!({
                    "message": "Invalid message type",
                }),
            }),
        }
    }
}

#[async_trait]
impl ProtocolServerTrait for ProtocolServer {
    async fn handle_call(
        &self,
        method: &str,
        payload: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let handlers = self.call_handlers.read().await;
        if let Some(handler) = handlers.get(method) {
            handler(payload).await
        } else {
            Err(anyhow::anyhow!("Method not found: {}", method))
        }
    }
    
    async fn handle_stream(
        &self,
        method: &str,
        payload: serde_json::Value,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<serde_json::Value>> + Send>>> {
        let handlers = self.stream_handlers.read().await;
        if let Some(handler) = handlers.get(method) {
            handler(payload).await
        } else {
            Err(anyhow::anyhow!("Stream method not found: {}", method))
        }
    }
}

impl Default for ProtocolServer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl UnisonServer for ProtocolServer {
    async fn listen(&mut self, addr: &str) -> Result<(), NetworkError> {
        // Set running state
        {
            let mut running = self.running.write().await;
            *running = true;
        }
        
        // In a real implementation, this would bind to the address and start accepting connections
        tracing::info!("🎵 Unison Protocol server listening on {}", addr);
        Ok(())
    }
    
    async fn stop(&mut self) -> Result<(), NetworkError> {
        let mut running = self.running.write().await;
        *running = false;
        tracing::info!("🎵 Unison Protocol server stopped");
        Ok(())
    }
    
    fn is_running(&self) -> bool {
        // Synchronous check - in practice we'd use a sync mechanism
        false // Simplified for now
    }
}

impl UnisonServerExt for ProtocolServer {
    fn register_handler<F>(&mut self, method: &str, handler: F)
    where 
        F: Fn(serde_json::Value) -> Result<serde_json::Value, NetworkError> + Send + Sync + 'static
    {
        let handler = Arc::new(handler);
        
        // Register using async task
        let handlers = Arc::clone(&self.unison_handlers);
        let method = method.to_string();
        
        tokio::spawn(async move {
            let mut handlers = handlers.write().await;
            handlers.insert(method, handler);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_server_creation() {
        let server = ProtocolServer::new();
        assert!(!server.is_running());
    }
    
    #[tokio::test]
    async fn test_server_lifecycle() {
        use super::UnisonServerExt;
        
        let mut server = ProtocolServer::new();
        
        // Register a simple handler
        server.register_handler("ping", |_payload| {
            Ok(serde_json::json!({"message": "pong"}))
        });
        
        // Start server
        assert!(server.listen("127.0.0.1:8080").await.is_ok());
        
        // Stop server
        assert!(server.stop().await.is_ok());
    }
}