# Unison Protocol WebSocketメッセージングガイド

[English](../en/websocket-messaging.md) | **日本語**

## 概要

Unison Protocolは、KDL（KDL Document Language）ベースの型安全なメッセージング仕様です。このガイドでは、WebSocketを使用したリアルタイム通信での実装例を示します。

## Unison Protocolの特徴

### 型安全性
- KDLスキーマによる厳密な型定義
- コンパイル時の型チェック
- ランタイム検証機能

### 相互運用性
- 言語に依存しない仕様
- JSON/MessagePack/他のシリアライゼーション形式に対応
- 複数のトランスポート層に対応（QUIC優先、WebSocketフォールバック）

### アダプティブトランスポート
- **QUIC優先**: 利用可能な環境では高性能なQUICを使用
- **WebSocketフォールバック**: QUIC非対応環境では自動的にWebSocketに切り替え
- **透過的な切り替え**: アプリケーションレベルでは同一のAPIを提供

## メッセージスキーマ定義

### KDLスキーマ例

```kdl
protocol "messaging-system" version="1.0.0" {
    namespace "example.messaging"
    description "リアルタイムメッセージング プロトコル"
    
    // アダプティブトランスポート設定
    transport "adaptive" {
        primary "quic" {
            version "1.0"
            encryption "tls1.3"
            multiplexing true
            connection_migration true
            detection_timeout_ms 5000
        }
        fallback "websocket" {
            version "13" // RFC 6455
            subprotocol "unison-messaging-v1"
            compression true
            heartbeat_interval_ms 30000
        }
        auto_negotiation true
        preference_caching true
    }
    
    // チャットメッセージ
    message "ChatMessage" {
        description "チャット風のメッセージ交換"
        field "user_name" type="string" required=true description="送信者名"
        field "content" type="string" required=true description="メッセージ内容"
        field "timestamp" type="timestamp" required=true description="送信時刻"
        field "message_id" type="string" required=true description="メッセージID"
        field "room" type="string" required=false default="general" description="チャットルーム"
    }
    
    // システム通知
    message "SystemNotification" {
        description "システム通知メッセージ"
        field "type" type="string" required=true description="通知タイプ（info/warning/error）"
        field "title" type="string" required=true description="通知タイトル"
        field "message" type="string" required=true description="通知内容"
        field "timestamp" type="timestamp" required=true description="通知時刻"
        field "auto_dismiss" type="boolean" required=false default=true description="自動消去するか"
    }
    
    // カスタムデータ交換
    message "CustomData" {
        description "自由なデータ交換用"
        field "data_type" type="string" required=true description="データタイプ"
        field "payload" type="json" required=true description="JSON形式のデータ"
        field "sender" type="string" required=false description="送信者"
        field "timestamp" type="timestamp" required=true description="送信時刻"
    }
    
    // メッセージング サービス
    service "MessagingService" {
        description "リアルタイム メッセージング サービス"
        
        method "send_chat" {
            description "チャットメッセージを送信"
            request {
                field "user_name" type="string" required=true
                field "content" type="string" required=true
                field "room" type="string" required=false default="general"
            }
            response {
                field "message_id" type="string" required=true
                field "timestamp" type="timestamp" required=true
                field "status" type="string" required=true
            }
        }
        
        method "send_custom_data" {
            description "カスタムデータを送信"
            request {
                field "data_type" type="string" required=true
                field "payload" type="json" required=true
                field "target_users" type="array" required=false description="送信対象ユーザー"
            }
            response {
                field "data_id" type="string" required=true
                field "delivered_count" type="number" required=true
                field "timestamp" type="timestamp" required=true
            }
        }
    }
    
    // リアルタイム通知ストリーム
    stream "NotificationStream" {
        description "リアルタイム通知ストリーム"
        
        event "chat_message" {
            description "新しいチャットメッセージ"
            field "message" type="ChatMessage" required=true
        }
        
        event "system_notification" {
            description "システム通知"
            field "notification" type="SystemNotification" required=true
        }
        
        event "custom_data" {
            description "カスタムデータ通知"
            field "data" type="CustomData" required=true
        }
    }
}
```

## 実装例

### Rust実装（アダプティブトランスポート）

```rust
use axum::{
    extract::{ws::WebSocket, WebSocketUpgrade},
    response::Response,
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::{Arc, LazyLock};
use std::time::Duration;
use tokio::sync::{broadcast, Mutex};
use tokio::time::timeout;
use uuid::Uuid;
use tracing::{info, warn, error};

// アダプティブトランスポート検出
#[derive(Debug, Clone, PartialEq)]
pub enum TransportType {
    Quic,
    WebSocket,
}

pub struct TransportDetector;

impl TransportDetector {
    pub async fn detect_optimal_transport() -> TransportType {
        // QUIC接続を試行（5秒でタイムアウト）
        match timeout(Duration::from_millis(5000), Self::test_quic_connection()).await {
            Ok(Ok(())) => {
                info!("✅ QUIC transport is available and optimal");
                TransportType::Quic
            }
            Ok(Err(e)) => {
                warn!("⚠️ QUIC test failed, falling back to WebSocket: {}", e);
                TransportType::WebSocket
            }
            Err(_) => {
                warn!("⏰ QUIC detection timed out, falling back to WebSocket");
                TransportType::WebSocket
            }
        }
    }
    
    async fn test_quic_connection() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // QUIC利用可能性テスト（実装は環境依存）
        // 実際の実装では、特定のQUICエンドポイントへの接続を試行
        #[cfg(feature = "quic")]
        {
            // quinn crateなどを使用したQUIC接続テスト
            let endpoint = quinn::Endpoint::client("[::]:0".parse()?)?;
            let connection = endpoint.connect("[::1]:4433".parse()?, "localhost")?;
            let _conn = connection.await?;
            Ok(())
        }
        #[cfg(not(feature = "quic"))]
        {
            Err("QUIC feature not enabled".into())
        }
    }
}

// WebSocket接続管理
pub struct ConnectionManager {
    connections: Arc<Mutex<HashMap<String, broadcast::Sender<String>>>>,
    message_tx: broadcast::Sender<String>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        let (message_tx, _) = broadcast::channel(1024);
        Self {
            connections: Arc::new(Mutex::new(HashMap::new())),
            message_tx,
        }
    }

    pub async fn broadcast_message(&self, message: &str) -> usize {
        let connections = self.connections.lock().await;
        let mut sent_count = 0;
        
        for (connection_id, tx) in connections.iter() {
            if tx.send(message.to_string()).is_ok() {
                sent_count += 1;
            }
        }
        
        sent_count
    }
}

static CONNECTION_MANAGER: LazyLock<ConnectionManager> = LazyLock::new(|| ConnectionManager::new());

// WebSocketハンドラー
pub async fn websocket_handler(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(handle_websocket_connection)
}

async fn handle_websocket_connection(socket: WebSocket) {
    let connection_id = Uuid::new_v4().to_string();
    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = broadcast::channel::<String>(64);

    // 接続をマネージャーに登録
    CONNECTION_MANAGER.add_connection(connection_id.clone(), tx).await;

    // 接続確立メッセージを送信
    let welcome_message = json!({
        "type": "connection_established",
        "connection_id": connection_id,
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "protocol_version": "1.0.0"
    });

    let _ = sender.send(axum::extract::ws::Message::Text(welcome_message.to_string().into())).await;

    // メッセージ処理ループ
    tokio::select! {
        _ = async {
            while let Ok(message) = rx.recv().await {
                if sender.send(axum::extract::ws::Message::Text(message.into())).await.is_err() {
                    break;
                }
            }
        } => {},
        _ = async {
            while let Some(msg) = receiver.next().await {
                if let Ok(axum::extract::ws::Message::Text(text)) = msg {
                    let _ = handle_incoming_message(&connection_id, &text).await;
                }
            }
        } => {}
    }

    // 接続をマネージャーから削除
    CONNECTION_MANAGER.remove_connection(&connection_id).await;
}

async fn handle_incoming_message(connection_id: &str, message: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let parsed_message: Value = serde_json::from_str(message)?;
    
    match parsed_message.get("type").and_then(|t| t.as_str()) {
        Some("chat_message") => handle_chat_message(connection_id, &parsed_message).await?,
        Some("custom_data") => handle_custom_data(connection_id, &parsed_message).await?,
        Some("ping") => handle_ping(connection_id, &parsed_message).await?,
        _ => {
            // 未知のメッセージタイプのエラー処理
        }
    }
    
    Ok(())
}
```

### JavaScript/TypeScript実装

```typescript
interface ChatMessage {
  type: 'chat_message';
  user_name: string;
  content: string;
  room?: string;
  timestamp?: string;
  message_id?: string;
}

interface CustomData {
  type: 'custom_data';
  data_type: string;
  payload: any;
  sender?: string;
  timestamp?: string;
}

interface SystemNotification {
  type: 'system_notification';
  notification_type: 'info' | 'warning' | 'error';
  title: string;
  message: string;
  timestamp: string;
  auto_dismiss?: boolean;
}

type Message = ChatMessage | CustomData | SystemNotification;

class UnisonAdaptiveClient {
  private ws: WebSocket | null = null;
  private quicConnection: any = null; // QUIC接続（環境依存）
  private messageHandlers = new Map<string, (message: any) => void>();
  private currentTransport: 'quic' | 'websocket' = 'websocket';

  constructor(private baseUrl: string) {}

  async connect(): Promise<void> {
    // アダプティブトランスポート検出
    this.currentTransport = await this.detectOptimalTransport();
    
    if (this.currentTransport === 'quic') {
      return this.connectQuic();
    } else {
      return this.connectWebSocket();
    }
  }
  
  private async detectOptimalTransport(): Promise<'quic' | 'websocket'> {
    try {
      // QUIC接続テスト（タイムアウト5秒）
      const timeoutPromise = new Promise((_, reject) => 
        setTimeout(() => reject(new Error('QUIC detection timeout')), 5000)
      );
      
      // 簡易的なQUIC利用可能性チェック
      // 実際の実装では、WebTransport APIやQUIC-specific endpointを使用
      if ('WebTransport' in window) {
        try {
          const testTransport = new WebTransport(`https://${this.baseUrl.replace('ws://', '').replace('wss://', '')}:443`);
          await Promise.race([testTransport.ready, timeoutPromise]);
          testTransport.close();
          console.log('✅ QUIC transport detected and available');
          return 'quic';
        } catch (error) {
          console.warn('⚠️ QUIC test failed:', error);
        }
      }
      
      console.log('📡 Falling back to WebSocket transport');
      return 'websocket';
    } catch (error) {
      console.warn('🔄 Transport detection failed, using WebSocket:', error);
      return 'websocket';
    }
  }
  
  private connectQuic(): Promise<void> {
    return new Promise(async (resolve, reject) => {
      try {
        // WebTransport (QUIC over HTTP/3) を使用
        if ('WebTransport' in window) {
          const transport = new (window as any).WebTransport(`https://${this.baseUrl.replace('ws://', '').replace('wss://', '')}:443`);
          await transport.ready;
          
          this.quicConnection = transport;
          console.log('✅ QUIC connection established');
          
          // QUIC用のメッセージハンドリング
          this.setupQuicMessageHandling(transport);
          resolve();
        } else {
          throw new Error('WebTransport not supported');
        }
      } catch (error) {
        console.warn('⚠️ QUIC connection failed, falling back to WebSocket:', error);
        this.currentTransport = 'websocket';
        this.connectWebSocket().then(resolve).catch(reject);
      }
    });
  }
  
  private connectWebSocket(): Promise<void> {
    return new Promise((resolve, reject) => {
      const wsUrl = this.baseUrl.startsWith('http') 
        ? this.baseUrl.replace('http', 'ws') + '/ws'
        : this.baseUrl;
      
      this.ws = new WebSocket(wsUrl, 'unison-messaging-v1');
      
      this.ws.onopen = () => {
        console.log('✅ WebSocket connection established');
        resolve();
      };
      
      this.ws.onerror = (error) => {
        console.error('❌ WebSocket接続エラー:', error);
        reject(error);
      };
      
      this.ws.onmessage = (event) => {
        try {
          const message = JSON.parse(event.data);
          this.handleMessage(message);
        } catch (error) {
          console.error('メッセージ解析エラー:', error);
        }
      };
      
      this.ws.onclose = () => {
        console.log('📴 WebSocket接続が閉じられました');
      };
    });
  }

  private handleMessage(message: any) {
    const handler = this.messageHandlers.get(message.type);
    if (handler) {
      handler(message);
    } else {
      console.warn('未処理のメッセージタイプ:', message.type);
    }
  }

  onMessage<T extends Message>(type: T['type'], handler: (message: T) => void) {
    this.messageHandlers.set(type, handler);
  }

  sendMessage(message: Message) {
    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(message));
    } else {
      console.error('WebSocket接続がありません');
    }
  }

  sendChatMessage(userName: string, content: string, room = 'general') {
    const message: ChatMessage = {
      type: 'chat_message',
      user_name: userName,
      content,
      room
    };
    this.sendMessage(message);
  }

  sendCustomData(dataType: string, payload: any, sender?: string) {
    const message: CustomData = {
      type: 'custom_data',
      data_type: dataType,
      payload,
      sender
    };
    this.sendMessage(message);
  }

  disconnect() {
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
  }
}

// 使用例
async function example() {
  const client = new UnisonWebSocketClient('ws://localhost:8080/ws');
  
  // メッセージハンドラーを設定
  client.onMessage('chat_message', (message) => {
    console.log(`💬 ${message.user_name}: ${message.content}`);
  });
  
  client.onMessage('system_notification', (message) => {
    console.log(`🔔 ${message.title}: ${message.message}`);
  });
  
  // 接続
  await client.connect();
  
  // メッセージ送信
  client.sendChatMessage('開発者A', 'こんにちは！', 'development');
  
  client.sendCustomData('progress_update', {
    task_id: 'task_001',
    progress: 0.75,
    status: 'processing'
  }, 'task_service');
}
```

### Python実装

```python
import asyncio
import websockets
import json
from typing import Dict, Any, Callable, Optional
from datetime import datetime

class UnisonWebSocketClient:
    def __init__(self, uri: str):
        self.uri = uri
        self.websocket = None
        self.message_handlers: Dict[str, Callable] = {}
    
    async def connect(self):
        """WebSocketサーバーに接続"""
        self.websocket = await websockets.connect(self.uri)
        print("✅ WebSocket接続が確立されました")
        
        # メッセージ受信ループを開始
        await self._listen()
    
    async def _listen(self):
        """メッセージ受信ループ"""
        try:
            async for message in self.websocket:
                try:
                    data = json.loads(message)
                    await self._handle_message(data)
                except json.JSONDecodeError:
                    print(f"❌ JSON解析エラー: {message}")
        except websockets.exceptions.ConnectionClosed:
            print("📴 WebSocket接続が閉じられました")
    
    async def _handle_message(self, message: Dict[str, Any]):
        """受信メッセージの処理"""
        message_type = message.get('type')
        handler = self.message_handlers.get(message_type)
        
        if handler:
            await handler(message)
        else:
            print(f"⚠️ 未処理のメッセージタイプ: {message_type}")
    
    def on_message(self, message_type: str, handler: Callable):
        """メッセージハンドラーを登録"""
        self.message_handlers[message_type] = handler
    
    async def send_message(self, message: Dict[str, Any]):
        """メッセージを送信"""
        if self.websocket:
            await self.websocket.send(json.dumps(message))
        else:
            print("❌ WebSocket接続がありません")
    
    async def send_chat_message(self, user_name: str, content: str, room: str = 'general'):
        """チャットメッセージを送信"""
        message = {
            'type': 'chat_message',
            'user_name': user_name,
            'content': content,
            'room': room,
            'timestamp': datetime.utcnow().isoformat() + 'Z'
        }
        await self.send_message(message)
    
    async def send_custom_data(self, data_type: str, payload: Any, sender: Optional[str] = None):
        """カスタムデータを送信"""
        message = {
            'type': 'custom_data',
            'data_type': data_type,
            'payload': payload,
            'sender': sender,
            'timestamp': datetime.utcnow().isoformat() + 'Z'
        }
        await self.send_message(message)
    
    async def disconnect(self):
        """接続を切断"""
        if self.websocket:
            await self.websocket.close()

# 使用例
async def main():
    client = UnisonWebSocketClient('ws://localhost:8080/ws')
    
    # メッセージハンドラーを設定
    async def handle_chat_message(message):
        print(f"💬 {message['user_name']}: {message['content']}")
    
    async def handle_system_notification(message):
        print(f"🔔 {message['title']}: {message['message']}")
    
    client.on_message('chat_message', handle_chat_message)
    client.on_message('system_notification', handle_system_notification)
    
    try:
        # 接続と通信
        await client.connect()
        
        # メッセージ送信
        await client.send_chat_message('開発者A', 'こんにちは！', 'development')
        
        await client.send_custom_data('progress_update', {
            'task_id': 'task_001',
            'progress': 0.75,
            'status': 'processing'
        }, 'task_service')
        
    except Exception as e:
        print(f"❌ エラー: {e}")
    finally:
        await client.disconnect()

if __name__ == "__main__":
    asyncio.run(main())
```

## テスト用HTMLページ

```html
<!DOCTYPE html>
<html lang="ja">
<head>
    <meta charset="UTF-8">
    <title>Unison Protocol WebSocketテスト</title>
</head>
<body>
    <div id="messages"></div>
    <input type="text" id="messageInput" placeholder="メッセージを入力">
    <button onclick="sendMessage()">送信</button>

    <script>
        const ws = new WebSocket('ws://localhost:8080/ws');
        
        ws.onopen = () => console.log('✅ 接続確立');
        ws.onclose = () => console.log('📴 接続切断');
        
        ws.onmessage = (event) => {
            const message = JSON.parse(event.data);
            document.getElementById('messages').innerHTML += 
                `<div>${JSON.stringify(message, null, 2)}</div>`;
        };
        
        function sendMessage() {
            const content = document.getElementById('messageInput').value;
            if (content) {
                const message = {
                    type: 'chat_message',
                    user_name: 'テストユーザー',
                    content: content,
                    room: 'test'
                };
                ws.send(JSON.stringify(message));
                document.getElementById('messageInput').value = '';
            }
        }
    </script>
</body>
</html>
```

## ベストプラクティス

### 1. エラーハンドリング
```rust
// 適切なエラー応答
let error_response = json!({
    "type": "error",
    "error_code": "validation_failed",
    "message": "必須フィールドが不足しています",
    "details": {
        "missing_fields": ["user_name", "content"]
    },
    "timestamp": chrono::Utc::now().to_rfc3339()
});
```

### 2. 型安全性の確保
```typescript
// TypeScriptでの型定義例
interface MessageBase {
  type: string;
  timestamp?: string;
}

interface ChatMessage extends MessageBase {
  type: 'chat_message';
  user_name: string;
  content: string;
  room?: string;
}

// 型ガード関数
function isChatMessage(message: any): message is ChatMessage {
  return message.type === 'chat_message' && 
         typeof message.user_name === 'string' &&
         typeof message.content === 'string';
}
```

### 3. パフォーマンス最適化
- メッセージのバッチ処理
- 接続プールの効率的な管理
- 適切なタイムアウト設定
- メモリリークの防止

## 参考資料

- [KDL (KDL Document Language) 仕様](https://kdl.dev/)
- [WebSocket RFC 6455](https://tools.ietf.org/html/rfc6455)
- [JSON Schema](https://json-schema.org/)

---

**最終更新**: 2024年1月 | **バージョン**: 1.0.0