//! CGPベースのコンテキスト定義
//! 
//! Context-Generic Programmingを使用して、Unison Protocolの
//! モジュラーで拡張可能なアーキテクチャを実現します。

pub mod handlers;
pub mod adapter;

// Re-export key types
pub use handlers::{HandlerRegistry, Handler, MessageDispatcher};
pub use adapter::{QuicTransportAdapter, ServiceRegistryAdapter};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::network::{NetworkError, ProtocolMessage};

// ========================================
// Core Components
// ========================================

/// プロトコルメッセージを扱うコンテキスト
pub trait HasProtocolMessage {
    type Message: Clone + Send + Sync;
    
    fn message(&self) -> &Self::Message;
}

/// トランスポート層を持つコンテキスト
pub trait HasTransport {
    type Transport: TransportLayer;
    
    fn transport(&self) -> &Self::Transport;
}

/// サービスレジストリを持つコンテキスト
pub trait HasServiceRegistry {
    type Registry: ServiceRegistry;
    
    fn registry(&self) -> &Self::Registry;
}

/// エラーハンドリングコンテキスト
pub trait HasErrorHandler {
    type Error: std::error::Error + Send + Sync + 'static;
    
    fn handle_error(&self, error: Self::Error);
}

// ========================================
// Transport Layer Traits
// ========================================

/// 汎用トランスポート層トレイト
#[allow(async_fn_in_trait)]
pub trait TransportLayer: Send + Sync {
    type Message;
    type Error;
    
    async fn send(&self, message: Self::Message) -> Result<(), Self::Error>;
    async fn receive(&self) -> Result<Self::Message, Self::Error>;
    async fn connect(&self, url: &str) -> Result<(), Self::Error>;
    async fn disconnect(&self) -> Result<(), Self::Error>;
    fn is_connected(&self) -> bool;
}

// ========================================
// Service Registry Traits  
// ========================================

/// サービスレジストリトレイト
#[allow(async_fn_in_trait)]
pub trait ServiceRegistry: Send + Sync {
    type Service;
    type Error;
    
    async fn register(&self, name: String, service: Self::Service) -> Result<(), Self::Error>;
    async fn get(&self, name: &str) -> Option<Self::Service>;
    async fn list(&self) -> Vec<String>;
    async fn remove(&self, name: &str) -> Result<(), Self::Error>;
}

// ========================================
// Handler Components
// ========================================

/// メッセージハンドラーコンポーネント
pub trait HasMessageHandler {
    type Handler: MessageHandler;
    
    fn handler(&self) -> &Self::Handler;
}

/// メッセージハンドラートレイト
#[allow(async_fn_in_trait)]
pub trait MessageHandler: Send + Sync {
    type Input;
    type Output;
    type Error;
    
    async fn handle(&self, input: Self::Input) -> Result<Self::Output, Self::Error>;
}

// ========================================
// Unified Context
// ========================================

/// Unison Protocol統合コンテキスト
pub trait UnisonContext: 
    HasProtocolMessage +
    HasTransport +
    HasServiceRegistry +
    HasErrorHandler +
    HasMessageHandler +
    Send + Sync
{
}

// ========================================
// Concrete Implementation
// ========================================

/// CGPベースのプロトコルコンテキスト実装
pub struct CgpProtocolContext<T, R, H> 
where
    T: TransportLayer,
    R: ServiceRegistry,
    H: MessageHandler,
{
    transport: Arc<T>,
    registry: Arc<RwLock<R>>,
    handler: Arc<H>,
    current_message: Option<ProtocolMessage>,
}

impl<T, R, H> CgpProtocolContext<T, R, H>
where
    T: TransportLayer,
    R: ServiceRegistry,
    H: MessageHandler,
{
    pub fn new(transport: T, registry: R, handler: H) -> Self {
        Self {
            transport: Arc::new(transport),
            registry: Arc::new(RwLock::new(registry)),
            handler: Arc::new(handler),
            current_message: None,
        }
    }
    
    /// トランスポート層への公開アクセス
    pub fn transport(&self) -> &T {
        &self.transport
    }
    
    /// レジストリへの公開アクセス
    pub fn registry(&self) -> &Arc<RwLock<R>> {
        &self.registry
    }
    
    /// ハンドラーへの公開アクセス
    pub fn handler(&self) -> &H {
        &self.handler
    }
}

// Component implementations
impl<T, R, H> HasTransport for CgpProtocolContext<T, R, H>
where
    T: TransportLayer,
    R: ServiceRegistry,
    H: MessageHandler,
{
    type Transport = T;
    
    fn transport(&self) -> &Self::Transport {
        &self.transport
    }
}

impl<T, R, H> HasProtocolMessage for CgpProtocolContext<T, R, H>
where
    T: TransportLayer,
    R: ServiceRegistry,
    H: MessageHandler,
{
    type Message = Option<ProtocolMessage>;
    
    fn message(&self) -> &Self::Message {
        &self.current_message
    }
}

impl<T, R, H> HasMessageHandler for CgpProtocolContext<T, R, H>
where
    T: TransportLayer,
    R: ServiceRegistry,
    H: MessageHandler,
{
    type Handler = H;
    
    fn handler(&self) -> &Self::Handler {
        &self.handler
    }
}

// ========================================
// Builder Pattern with CGP
// ========================================

/// コンテキストビルダー
pub struct UnisonContextBuilder<T, R, H> {
    transport: Option<T>,
    registry: Option<R>,
    handler: Option<H>,
}

impl<T, R, H> UnisonContextBuilder<T, R, H> {
    pub fn new() -> Self {
        Self {
            transport: None,
            registry: None,
            handler: None,
        }
    }
    
    pub fn with_transport(mut self, transport: T) -> Self {
        self.transport = Some(transport);
        self
    }
    
    pub fn with_registry(mut self, registry: R) -> Self {
        self.registry = Some(registry);
        self
    }
    
    pub fn with_handler(mut self, handler: H) -> Self {
        self.handler = Some(handler);
        self
    }
    
    pub fn build(self) -> Result<CgpProtocolContext<T, R, H>, String>
    where
        T: TransportLayer,
        R: ServiceRegistry,
        H: MessageHandler,
    {
        let transport = self.transport.ok_or("Transport not set")?;
        let registry = self.registry.ok_or("Registry not set")?;
        let handler = self.handler.ok_or("Handler not set")?;
        
        Ok(CgpProtocolContext::new(transport, registry, handler))
    }
}

// ========================================
// Error Types
// ========================================

/// CGPコンテキストエラー
#[derive(Debug, thiserror::Error)]
pub enum ContextError {
    #[error("Transport error: {0}")]
    Transport(String),
    
    #[error("Registry error: {0}")]
    Registry(String),
    
    #[error("Handler error: {0}")]
    Handler(String),
    
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("Network error: {0}")]
    Network(#[from] NetworkError),
}

impl<T, R, H> HasErrorHandler for CgpProtocolContext<T, R, H>
where
    T: TransportLayer,
    R: ServiceRegistry,
    H: MessageHandler,
{
    type Error = ContextError;
    
    fn handle_error(&self, error: Self::Error) {
        tracing::error!("Context error: {}", error);
    }
}