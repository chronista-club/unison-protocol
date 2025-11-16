//! エラー型定義

use thiserror::Error;

/// Unison Agentのエラー型
#[derive(Debug, Error)]
pub enum AgentError {
    /// Claude Agent SDKエラー
    #[error("Claude Agent SDK error: {0}")]
    ClaudeAgent(String),

    /// 通信エラー
    #[error("Communication error: {0}")]
    Communication(String),

    /// 設定エラー
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// その他のエラー
    #[error("Other error: {0}")]
    Other(#[from] anyhow::Error),
}

/// Result型のエイリアス
pub type Result<T> = std::result::Result<T, AgentError>;
