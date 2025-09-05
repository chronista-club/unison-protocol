use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use futures_util::Stream;
use std::pin::Pin;
use std::time::SystemTime;
use thiserror::Error;
use tokio::sync::mpsc;

pub mod client;
pub mod server;
pub mod quic;
pub mod service;

pub use client::ProtocolClient;
pub use server::ProtocolServer;
pub use quic::{QuicClient, QuicServer, UnisonStream};
pub use service::{Service, RealtimeService, ServiceConfig, ServicePriority, ServiceStats, UnisonService};

/// Network errors for Unison Protocol
#[derive(Error, Debug)]
pub enum NetworkError {
    #[error("Connection error: {0}")]
    Connection(String),
    #[error("Protocol error: {0}")]
    Protocol(String),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("QUIC error: {0}")]
    Quic(String),
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
    StreamError,
    // Bidirectional streaming types
    BidirectionalStream,
    StreamSend,
    StreamReceive,
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
    
    /// Register a stream handler for a specific method
    fn register_stream_handler<F>(&mut self, method: &str, handler: F)
    where 
        F: Fn(serde_json::Value) -> Pin<Box<dyn Stream<Item = Result<serde_json::Value, NetworkError>> + Send>> + Send + Sync + 'static;
    
    /// Register a SystemStream handler for bidirectional streaming
    fn register_system_stream_handler<F>(&mut self, method: &str, handler: F)
    where 
        F: Fn(serde_json::Value, Box<dyn SystemStream>) -> Pin<Box<dyn futures_util::Future<Output = Result<(), NetworkError>> + Send>> + Send + Sync + 'static;
}

/// SystemStream - Bidirectional stream trait for QUIC
#[async_trait]
pub trait SystemStream: Send + Sync {
    /// Send data on the stream
    async fn send(&mut self, data: serde_json::Value) -> Result<(), NetworkError>;
    
    /// Receive data from the stream
    async fn receive(&mut self) -> Result<serde_json::Value, NetworkError>;
    
    /// Check if stream is still active
    fn is_active(&self) -> bool;
    
    /// Close the stream
    async fn close(&mut self) -> Result<(), NetworkError>;
    
    /// Get stream metadata
    fn get_handle(&self) -> StreamHandle;
}

/// Stream handle for managing bidirectional streams
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamHandle {
    pub stream_id: u64,
    pub method: String,
    pub created_at: std::time::SystemTime,
}

/// Extended client trait with SystemStream support
#[async_trait]
pub trait UnisonClientExt: UnisonClient {
    /// Start a bidirectional SystemStream
    async fn start_system_stream(&mut self, method: &str, payload: serde_json::Value) -> Result<Box<dyn SystemStream>, NetworkError>;
    
    /// List active SystemStreams
    async fn list_system_streams(&self) -> Result<Vec<StreamHandle>, NetworkError>;
    
    /// Close a specific SystemStream by ID
    async fn close_system_stream(&mut self, stream_id: u64) -> Result<(), NetworkError>;
}