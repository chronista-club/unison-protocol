use anyhow::Result;
use serde::{Deserialize, Serialize};
use futures_util::Stream;
use std::pin::Pin;
use thiserror::Error;

pub mod client;
pub mod server;
pub mod quic;
pub mod service;

pub use client::ProtocolClient;
pub use server::ProtocolServer;
pub use quic::{QuicClient, QuicServer, UnisonStream};
pub use service::{Service, RealtimeService, ServiceConfig, ServicePriority, ServiceStats, UnisonService};

/// Unison Protocolのネットワークエラー
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

/// プロトコルメッセージラッパー
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolMessage {
    pub id: u64,
    pub method: String,
    #[serde(rename = "type")]
    pub msg_type: MessageType,
    pub payload: serde_json::Value,
}

/// メッセージ種別
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageType {
    Request,
    Response,
    Stream,
    StreamData,
    StreamEnd,
    StreamError,
    // 双方向ストリーミング種別
    BidirectionalStream,
    StreamSend,
    StreamReceive,
    Error,
}

/// プロトコルエラー
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolError {
    pub code: i32,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

/// プロトコル呼び出し用クライアントトレイト
pub trait ProtocolClientTrait: Send + Sync {
    /// 単項RPC呼び出しの実行
    async fn call<TRequest, TResponse>(&self, method: &str, request: TRequest) -> Result<TResponse>
    where
        TRequest: Serialize + Send + Sync,
        TResponse: for<'de> Deserialize<'de>;
    
    /// ストリーミングRPC呼び出しの開始
    async fn stream<TRequest, TResponse>(
        &self,
        method: &str,
        request: TRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<TResponse>> + Send>>>
    where
        TRequest: Serialize + Send + Sync,
        TResponse: for<'de> Deserialize<'de> + Send + 'static;
}

/// プロトコルリクエスト処理用サーバートレイト
pub trait ProtocolServerTrait: Send + Sync {
    /// 単項RPC呼び出しの処理
    async fn handle_call(
        &self,
        method: &str,
        payload: serde_json::Value,
    ) -> Result<serde_json::Value>;
    
    /// ストリーミングRPC呼び出しの処理
    async fn handle_stream(
        &self,
        method: &str,
        payload: serde_json::Value,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<serde_json::Value>> + Send>>>;
}

/// Unison Protocolクライアントトレイト
pub trait UnisonClient: Send + Sync {
    /// Unisonサーバーへの接続
    async fn connect(&mut self, url: &str) -> Result<(), NetworkError>;
    
    /// リモートプロシージャ呼び出しの実行
    async fn call(&mut self, method: &str, payload: serde_json::Value) -> Result<serde_json::Value, NetworkError>;
    
    /// サーバーからの切断
    async fn disconnect(&mut self) -> Result<(), NetworkError>;
    
    /// クライアント接続状態の確認
    fn is_connected(&self) -> bool;
}

/// Unison Protocolサーバートレイト（dyn互換）
pub trait UnisonServer: Send + Sync {
    /// 接続の待ち受け開始
    async fn listen(&mut self, addr: &str) -> Result<(), NetworkError>;
    
    /// サーバーの停止
    async fn stop(&mut self) -> Result<(), NetworkError>;
    
    /// サーバー実行状態の確認
    fn is_running(&self) -> bool;
}

/// ハンドラー登録トレイト（ジェネリクスのためdyn非互換）
pub trait UnisonServerExt: UnisonServer {
    /// 特定メソッド用ハンドラーの登録
    fn register_handler<F>(&mut self, method: &str, handler: F)
    where 
        F: Fn(serde_json::Value) -> Result<serde_json::Value, NetworkError> + Send + Sync + 'static;
    
    /// 特定メソッド用ストリームハンドラーの登録
    fn register_stream_handler<F>(&mut self, method: &str, handler: F)
    where 
        F: Fn(serde_json::Value) -> Pin<Box<dyn Stream<Item = Result<serde_json::Value, NetworkError>> + Send>> + Send + Sync + 'static;
    
    /// 双方向ストリーミング用SystemStreamハンドラーの登録
    fn register_system_stream_handler<F>(&mut self, method: &str, handler: F)
    where 
        F: Fn(serde_json::Value, SystemStreamWrapper) -> Pin<Box<dyn futures_util::Future<Output = Result<(), NetworkError>> + Send>> + Send + Sync + 'static;
}

/// SystemStream - QUIC用双方向ストリームトレイト
pub trait SystemStream: Send + Sync {
    /// ストリームでのデータ送信
    async fn send(&mut self, data: serde_json::Value) -> Result<(), NetworkError>;
    
    /// ストリームからのデータ受信
    async fn receive(&mut self) -> Result<serde_json::Value, NetworkError>;
    
    /// ストリーム稼働状態の確認
    fn is_active(&self) -> bool;
    
    /// ストリームの終了
    async fn close(&mut self) -> Result<(), NetworkError>;
    
    /// ストリームメタデータの取得
    fn get_handle(&self) -> StreamHandle;
}

/// SystemStreamのenum wrapper - dyn互換性のため
pub enum SystemStreamWrapper {
    Quic(crate::network::quic::UnisonStream),
    Mock(crate::network::client::MockSystemStream),
}

impl SystemStreamWrapper {
    pub fn new_quic(stream: crate::network::quic::UnisonStream) -> Self {
        Self::Quic(stream)
    }
    
    pub fn new_mock(mock: crate::network::client::MockSystemStream) -> Self {
        Self::Mock(mock)
    }
}

impl SystemStream for SystemStreamWrapper {
    async fn send(&mut self, data: serde_json::Value) -> Result<(), NetworkError> {
        match self {
            Self::Quic(stream) => stream.send(data).await,
            Self::Mock(mock) => mock.send(data).await,
        }
    }
    
    async fn receive(&mut self) -> Result<serde_json::Value, NetworkError> {
        match self {
            Self::Quic(stream) => stream.receive().await,
            Self::Mock(mock) => mock.receive().await,
        }
    }
    
    fn is_active(&self) -> bool {
        match self {
            Self::Quic(stream) => stream.is_active(),
            Self::Mock(mock) => mock.is_active(),
        }
    }
    
    async fn close(&mut self) -> Result<(), NetworkError> {
        match self {
            Self::Quic(stream) => stream.close().await,
            Self::Mock(mock) => mock.close().await,
        }
    }
    
    fn get_handle(&self) -> StreamHandle {
        match self {
            Self::Quic(stream) => stream.get_handle(),
            Self::Mock(mock) => mock.get_handle(),
        }
    }
}

/// 双方向ストリーム管理用ストリームハンドル
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamHandle {
    pub stream_id: u64,
    pub method: String,
    pub created_at: std::time::SystemTime,
}

/// SystemStreamサポート付き拡張クライアントトレイト
pub trait UnisonClientExt: UnisonClient {
    /// 双方向SystemStreamの開始
    async fn start_system_stream(&mut self, method: &str, payload: serde_json::Value) -> Result<SystemStreamWrapper, NetworkError>;
    
    /// アクティブなSystemStreamの一覧
    async fn list_system_streams(&self) -> Result<Vec<StreamHandle>, NetworkError>;
    
    /// ID指定によるSystemStreamの終了
    async fn close_system_stream(&mut self, stream_id: u64) -> Result<(), NetworkError>;
}