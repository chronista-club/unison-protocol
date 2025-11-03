//! Claude Agentクライアント実装

use claude_agent_sdk::{ClaudeAgentOptions, ClaudeSDKClient, ContentBlock, Message};
use tracing::{debug, info};

use crate::error::{AgentError, Result};

/// Claude Agentとの対話型クライアント
pub struct InteractiveClient {
    inner: ClaudeSDKClient,
}

impl InteractiveClient {
    /// 新しいクライアントを作成
    pub async fn new() -> Result<Self> {
        let options = ClaudeAgentOptions::builder().build();

        let client = ClaudeSDKClient::new(options, None)
            .await
            .map_err(|e| AgentError::ClaudeAgent(e.to_string()))?;

        Ok(Self { inner: client })
    }

    /// メッセージを送信してレスポンスを取得
    pub async fn query(&mut self, message: &str) -> Result<Vec<String>> {
        info!("Sending message: {}", message);

        // メッセージを送信
        self.inner
            .send_message(message)
            .await
            .map_err(|e| AgentError::Communication(e.to_string()))?;

        debug!("Waiting for response...");
        let mut responses = Vec::new();

        // レスポンスを受信
        while let Some(msg_result) = self.inner.next_message().await {
            let msg = msg_result.map_err(|e| AgentError::Communication(e.to_string()))?;

            match msg {
                Message::Assistant { message, .. } => {
                    for block in &message.content {
                        if let ContentBlock::Text { text } = block {
                            debug!("Received text: {}", text);
                            responses.push(text.clone());
                        }
                    }
                }
                Message::Result { .. } => {
                    debug!("Conversation completed");
                    break;
                }
                _ => {}
            }
        }

        Ok(responses)
    }

    /// クライアントを閉じる
    pub async fn close(&mut self) -> Result<()> {
        info!("Closing client...");
        self.inner
            .close()
            .await
            .map_err(|e| AgentError::Communication(e.to_string()))?;
        Ok(())
    }
}
