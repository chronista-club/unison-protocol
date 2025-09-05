use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use futures_util::Stream;
use std::pin::Pin;
use thiserror::Error;

pub mod client;
pub mod server;
pub mod websocket;

pub use client::ProtocolClient;
pub use server::ProtocolServer;
pub use websocket::{WebSocketClient, WebSocketServer};

/// Network errors for Unison Protocol
#[derive(Error, Debug)]
pub enum NetworkError {
    #[error("Connection error: {0}")]
    Connection(String),
    #[error("Protocol error: {0}")]
    Protocol(String),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("WebSocket error: {0}")]
    WebSocket(String),
    #[error("Timeout error")]
    Timeout,
    #[error("Handler not found for method: {method}")]
    HandlerNotFound { method: String },
}

/// Protocol message wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolMessage {
    pub id: u64,
    pub method: String,
    #[serde(rename = "type")]
    pub msg_type: MessageType,
    pub payload: serde_json::Value,
}

/// Message type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageType {
    Request,
    Response,
    Stream,
    StreamData,
    StreamEnd,
    Error,
}

/// Protocol error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolError {
    pub code: i32,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

/// Client trait for making protocol calls
#[async_trait]
pub trait ProtocolClientTrait: Send + Sync {
    /// Make a unary RPC call
    async fn call<TRequest, TResponse>(&self, method: &str, request: TRequest) -> Result<TResponse>
    where
        TRequest: Serialize + Send + Sync,
        TResponse: for<'de> Deserialize<'de>;
    
    /// Start a streaming RPC call
    async fn stream<TRequest, TResponse>(
        &self,
        method: &str,
        request: TRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<TResponse>> + Send>>>
    where
        TRequest: Serialize + Send + Sync,
        TResponse: for<'de> Deserialize<'de> + Send + 'static;
}

/// Server trait for handling protocol requests
#[async_trait]
pub trait ProtocolServerTrait: Send + Sync {
    /// Handle a unary RPC call
    async fn handle_call(
        &self,
        method: &str,
        payload: serde_json::Value,
    ) -> Result<serde_json::Value>;
    
    /// Handle a streaming RPC call
    async fn handle_stream(
        &self,
        method: &str,
        payload: serde_json::Value,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<serde_json::Value>> + Send>>>;
}

/// Unison Protocol client trait
#[async_trait]
pub trait UnisonClient: Send + Sync {
    /// Connect to a Unison server
    async fn connect(&mut self, url: &str) -> Result<(), NetworkError>;
    
    /// Make a remote procedure call
    async fn call(&mut self, method: &str, payload: serde_json::Value) -> Result<serde_json::Value, NetworkError>;
    
    /// Disconnect from the server
    async fn disconnect(&mut self) -> Result<(), NetworkError>;
    
    /// Check if client is connected
    fn is_connected(&self) -> bool;
}

/// Unison Protocol server trait (dyn-compatible)
#[async_trait]
pub trait UnisonServer: Send + Sync {
    /// Start listening for connections
    async fn listen(&mut self, addr: &str) -> Result<(), NetworkError>;
    
    /// Stop the server
    async fn stop(&mut self) -> Result<(), NetworkError>;
    
    /// Check if server is running
    fn is_running(&self) -> bool;
}

/// Handler registration trait (non-dyn-compatible due to generics)
pub trait UnisonServerExt: UnisonServer {
    /// Register a handler for a specific method
    fn register_handler<F>(&mut self, method: &str, handler: F)
    where 
        F: Fn(serde_json::Value) -> Result<serde_json::Value, NetworkError> + Send + Sync + 'static;
}