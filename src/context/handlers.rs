//! CGPベースの拡張可能なハンドラー実装
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::network::{NetworkError, ProtocolMessage};

// ========================================
// Extensible Handler System
// ========================================

/// 拡張可能なメッセージディスパッチャー (Rust 2024対応)
pub trait MessageDispatcher {
    fn dispatch(
        &self,
        message: ProtocolMessage,
    ) -> impl std::future::Future<Output = Result<Value, NetworkError>> + Send;
}

/// メソッド別ハンドラーレジストリ
pub struct HandlerRegistry {
    handlers: Arc<RwLock<HashMap<String, Arc<dyn Handler>>>>,
}

/// 汎用ハンドラートレイト
pub trait Handler: Send + Sync {
    fn handle(
        &self,
        payload: Value,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Value, NetworkError>> + Send + '_>>;
}

// ========================================
// Concrete Handler Implementations
// ========================================

/// Pingハンドラー
pub struct PingHandler;

impl Handler for PingHandler {
    fn handle(
        &self,
        payload: Value,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Value, NetworkError>> + Send + '_>>
    {
        Box::pin(async move {
            let message = payload
                .get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("Hello");

            Ok(serde_json::json!({
                "message": format!("Pong: {}", message),
                "timestamp": chrono::Utc::now().to_rfc3339()
            }))
        })
    }
}

/// Echoハンドラー
pub struct EchoHandler;

impl Handler for EchoHandler {
    fn handle(
        &self,
        payload: Value,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Value, NetworkError>> + Send + '_>>
    {
        Box::pin(async move { Ok(payload) })
    }
}

/// サービス情報ハンドラー
pub struct ServiceInfoHandler {
    pub service_name: String,
    pub version: String,
}

impl Handler for ServiceInfoHandler {
    fn handle(
        &self,
        _payload: Value,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Value, NetworkError>> + Send + '_>>
    {
        let service_name = self.service_name.clone();
        let version = self.version.clone();
        Box::pin(async move {
            Ok(serde_json::json!({
                "service": service_name,
                "version": version,
                "capabilities": ["ping", "echo", "service_info"],
                "status": "healthy"
            }))
        })
    }
}

// ========================================
// Modular Handler Composition
// ========================================

/// CGPを使った合成可能なハンドラー
pub struct CompositeHandler {
    handlers: Vec<Box<dyn Handler>>,
}

impl Default for CompositeHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl CompositeHandler {
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
        }
    }

    pub fn add_handler(mut self, handler: Box<dyn Handler>) -> Self {
        self.handlers.push(handler);
        self
    }
}

impl Handler for CompositeHandler {
    fn handle(
        &self,
        payload: Value,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Value, NetworkError>> + Send + '_>>
    {
        Box::pin(async move {
            // 最初にマッチしたハンドラーを使用
            for handler in &self.handlers {
                if let Ok(result) = handler.handle(payload.clone()).await {
                    return Ok(result);
                }
            }

            Err(NetworkError::HandlerNotFound {
                method: "composite".to_string(),
            })
        })
    }
}

// ========================================
// Handler Registry Implementation
// ========================================

impl Default for HandlerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl HandlerRegistry {
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// ハンドラーを登録
    pub async fn register(&self, method: &str, handler: impl Handler + 'static) {
        let mut handlers = self.handlers.write().await;
        handlers.insert(method.to_string(), Arc::new(handler));
    }

    /// メソッドに対応するハンドラーを取得
    pub async fn get(&self, method: &str) -> Option<Arc<dyn Handler>> {
        let handlers = self.handlers.read().await;
        handlers.get(method).cloned()
    }

    /// 登録されているメソッド一覧を取得
    pub async fn list_methods(&self) -> Vec<String> {
        let handlers = self.handlers.read().await;
        handlers.keys().cloned().collect()
    }
}

impl MessageDispatcher for HandlerRegistry {
    async fn dispatch(&self, message: ProtocolMessage) -> Result<Value, NetworkError> {
        let handlers = self.handlers.read().await;

        if let Some(handler) = handlers.get(&message.method) {
            let payload_value = message.payload_as_value()?;
            handler.handle(payload_value).await
        } else {
            Err(NetworkError::HandlerNotFound {
                method: message.method.clone(),
            })
        }
    }
}

// HandlerRegistryをMessageHandlerとして実装
impl super::MessageHandler for HandlerRegistry {
    type Input = ProtocolMessage;
    type Output = Value;
    type Error = NetworkError;

    async fn handle(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        self.dispatch(input).await
    }
}

// ========================================
// CGP Handler Macros
// ========================================

/// ハンドラーを自動登録するマクロ
#[macro_export]
macro_rules! register_handlers {
    ($registry:expr, $($handler:expr),*) => {
        {
            $(
                $registry.register(
                    $handler.method(),
                    Box::new($handler)
                ).await;
            )*
        }
    };
}

// ========================================
// Stream Handler Support
// ========================================

/// ストリーミングハンドラー (Rust 2024対応)
pub trait StreamHandler: Send + Sync {
    type Item;

    fn handle_stream(
        &self,
        payload: Value,
    ) -> impl std::future::Future<
        Output = Result<Box<dyn futures_util::Stream<Item = Self::Item> + Send>, NetworkError>,
    > + Send;
}

/// 数値ストリームハンドラーの例
pub struct NumberStreamHandler {
    pub max: u32,
}

impl StreamHandler for NumberStreamHandler {
    type Item = Result<Value, NetworkError>;

    async fn handle_stream(
        &self,
        _payload: Value,
    ) -> Result<Box<dyn futures_util::Stream<Item = Self::Item> + Send>, NetworkError> {
        let max = self.max;
        let stream = async_stream::stream! {
            for i in 0..max {
                yield Ok(serde_json::json!({ "number": i }));
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        };

        Ok(Box::new(stream))
    }
}
