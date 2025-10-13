//! Unison Protocolのコア型と定義
//!
//! このモジュールは、すべてのUnison Protocol通信の基礎となる
//! 基本的な型と構造体を提供します。

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Unisonプロトコルの標準メッセージフォーマット
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnisonMessage {
    /// 一意のメッセージ識別子
    pub id: String,
    /// RPCメソッド名
    pub method: String,
    /// JSON形式のメソッドパラメータ
    pub payload: serde_json::Value,
    /// メッセージ作成タイムスタンプ
    pub timestamp: DateTime<Utc>,
    /// プロトコルバージョン
    #[serde(default = "default_version")]
    pub version: String,
}

/// Standard response format for Unison protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnisonResponse {
    /// Corresponding request message ID
    pub id: String,
    /// Operation success indicator
    pub success: bool,
    /// Response data as JSON
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<serde_json::Value>,
    /// Error message if operation failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Response creation timestamp
    pub timestamp: DateTime<Utc>,
    /// Protocol version
    #[serde(default = "default_version")]
    pub version: String,
}

/// Structured error information for Unison protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnisonError {
    /// Error code identifier
    pub code: String,
    /// Human-readable error message
    pub message: String,
    /// Additional error context
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
    /// Error occurrence timestamp
    pub timestamp: DateTime<Utc>,
}

/// Handshake request for establishing protocol compatibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandshakeRequest {
    /// Client protocol version
    pub protocol_version: String,
    /// Client application identifier
    pub client_name: String,
    /// Client application version
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_version: Option<String>,
    /// List of client-supported features
    #[serde(default)]
    pub supported_features: Vec<String>,
}

/// Handshake response from server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandshakeResponse {
    /// Server protocol version
    pub server_version: String,
    /// Server application identifier
    pub server_name: String,
    /// List of server-supported features
    pub supported_features: Vec<String>,
    /// Unique session identifier
    pub session_id: String,
    /// Heartbeat interval in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heartbeat_interval: Option<u64>,
}

/// Ping request for connection health check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PingRequest {
    /// Ping timestamp
    pub timestamp: DateTime<Utc>,
    /// Optional ping payload
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<String>,
}

/// Pong response for connection health check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PongResponse {
    /// Pong timestamp
    pub timestamp: DateTime<Utc>,
    /// Echo of ping payload
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<String>,
    /// Server current timestamp
    pub server_time: DateTime<Utc>,
}

impl UnisonMessage {
    /// Create a new Unison message
    pub fn new(method: impl Into<String>, payload: serde_json::Value) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            method: method.into(),
            payload,
            timestamp: Utc::now(),
            version: default_version(),
        }
    }
    
    /// Create a message with a specific ID
    pub fn with_id(id: impl Into<String>, method: impl Into<String>, payload: serde_json::Value) -> Self {
        Self {
            id: id.into(),
            method: method.into(),
            payload,
            timestamp: Utc::now(),
            version: default_version(),
        }
    }
}

impl UnisonResponse {
    /// Create a successful response
    pub fn success(id: impl Into<String>, payload: serde_json::Value) -> Self {
        Self {
            id: id.into(),
            success: true,
            payload: Some(payload),
            error: None,
            timestamp: Utc::now(),
            version: default_version(),
        }
    }
    
    /// Create an error response
    pub fn error(id: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            success: false,
            payload: None,
            error: Some(error.into()),
            timestamp: Utc::now(),
            version: default_version(),
        }
    }
    
    /// Create an empty success response
    pub fn empty_success(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            success: true,
            payload: None,
            error: None,
            timestamp: Utc::now(),
            version: default_version(),
        }
    }
}

impl UnisonError {
    /// Create a new Unison error
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            details: None,
            timestamp: Utc::now(),
        }
    }
    
    /// Create an error with additional details
    pub fn with_details(
        code: impl Into<String>, 
        message: impl Into<String>,
        details: serde_json::Value
    ) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            details: Some(details),
            timestamp: Utc::now(),
        }
    }
}

fn default_version() -> String {
    "1.0.0".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_unison_message_creation() {
        let msg = UnisonMessage::new("test_method", serde_json::json!({"key": "value"}));
        assert_eq!(msg.method, "test_method");
        assert_eq!(msg.version, "1.0.0");
        assert!(!msg.id.is_empty());
    }
    
    #[test]
    fn test_unison_response_success() {
        let resp = UnisonResponse::success("123", serde_json::json!({"result": "ok"}));
        assert_eq!(resp.id, "123");
        assert!(resp.success);
        assert!(resp.payload.is_some());
        assert!(resp.error.is_none());
    }
    
    #[test]
    fn test_unison_response_error() {
        let resp = UnisonResponse::error("123", "Something went wrong");
        assert_eq!(resp.id, "123");
        assert!(!resp.success);
        assert!(resp.payload.is_none());
        assert_eq!(resp.error.as_ref().unwrap(), "Something went wrong");
    }
    
    #[test]
    fn test_serialization_roundtrip() {
        let msg = UnisonMessage::new("ping", serde_json::json!({"message": "hello"}));
        let json = serde_json::to_string(&msg).unwrap();
        let deserialized: UnisonMessage = serde_json::from_str(&json).unwrap();
        
        assert_eq!(msg.id, deserialized.id);
        assert_eq!(msg.method, deserialized.method);
        assert_eq!(msg.payload, deserialized.payload);
        assert_eq!(msg.version, deserialized.version);
    }
}