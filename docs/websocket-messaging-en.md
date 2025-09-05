# Unison Protocol WebSocket Messaging Guide

**English** | [日本語](./websocket-messaging-ja.md)

## Overview

Unison Protocol is a type-safe messaging specification based on KDL (KDL Document Language). This guide demonstrates implementation examples for real-time communication using WebSocket over QUIC transport layer.

## Unison Protocol Features

### Type Safety
- Strict type definitions with KDL schemas
- Compile-time type checking
- Runtime validation capabilities

### Interoperability
- Language-agnostic specification
- Support for JSON/MessagePack/other serialization formats
- Multiple transport layer support (QUIC-preferred with WebSocket fallback)

### Adaptive Transport
- **QUIC Preferred**: High-performance QUIC transport when available
- **WebSocket Fallback**: Automatic fallback to WebSocket in unsupported environments
- **Transparent Switching**: Same API at application level regardless of transport

## Message Schema Definition

### KDL Schema Example

```kdl
protocol "messaging-system" version="1.0.0" {
    namespace "example.messaging"
    description "Real-time messaging protocol with adaptive transport"
    
    // Adaptive transport configuration
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
    
    // Chat Message
    message "ChatMessage" {
        description "Chat-style message exchange"
        field "user_name" type="string" required=true description="Sender name"
        field "content" type="string" required=true description="Message content"
        field "timestamp" type="timestamp" required=true description="Send time"
        field "message_id" type="string" required=true description="Message ID"
        field "room" type="string" required=false default="general" description="Chat room"
    }
    
    // System Notification
    message "SystemNotification" {
        description "System notification message"
        field "type" type="string" required=true description="Notification type (info/warning/error)"
        field "title" type="string" required=true description="Notification title"
        field "message" type="string" required=true description="Notification content"
        field "timestamp" type="timestamp" required=true description="Notification time"
        field "auto_dismiss" type="boolean" required=false default=true description="Auto dismiss flag"
    }
    
    // Custom Data Exchange
    message "CustomData" {
        description "Generic data exchange"
        field "data_type" type="string" required=true description="Data type identifier"
        field "payload" type="json" required=true description="JSON data payload"
        field "sender" type="string" required=false description="Sender identifier"
        field "timestamp" type="timestamp" required=true description="Send time"
    }
    
    // Messaging Service
    service "MessagingService" {
        description "Real-time messaging service over QUIC"
        
        method "send_chat" {
            description "Send chat message"
            request {
                field "user_name" type="string" required=true
                field "content" type="string" required=true
                field "room" type="string" required=false default="general"
            }
            response {
                field "message_id" type="string" required=true
                field "timestamp" type="timestamp" required=true
                field "status" type="string" required=true
                field "stream_id" type="number" required=true description="QUIC stream ID"
            }
        }
        
        method "send_custom_data" {
            description "Send custom data"
            request {
                field "data_type" type="string" required=true
                field "payload" type="json" required=true
                field "target_users" type="array" required=false description="Target users"
            }
            response {
                field "data_id" type="string" required=true
                field "delivered_count" type="number" required=true
                field "timestamp" type="timestamp" required=true
                field "stream_id" type="number" required=true description="QUIC stream ID"
            }
        }
    }
    
    // Real-time Notification Stream
    stream "NotificationStream" {
        description "Real-time notification stream over QUIC"
        transport_settings {
            priority "high"
            reliable true
            ordered false
        }
        
        event "chat_message" {
            description "New chat message"
            field "message" type="ChatMessage" required=true
        }
        
        event "system_notification" {
            description "System notification"
            field "notification" type="SystemNotification" required=true
        }
        
        event "custom_data" {
            description "Custom data notification"
            field "data" type="CustomData" required=true
        }
    }
}
```

## Implementation Examples

### Rust Implementation with QUIC

```rust
use axum::{
    extract::{ws::WebSocket, WebSocketUpgrade},
    response::Response,
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::{Arc, LazyLock};
use tokio::sync::{broadcast, Mutex};
use uuid::Uuid;
use quinn::{Connection, Endpoint};

// QUIC-enabled WebSocket Connection Manager
pub struct QuicConnectionManager {
    connections: Arc<Mutex<HashMap<String, QuicConnection>>>,
    endpoint: Endpoint,
}

pub struct QuicConnection {
    connection_id: String,
    quic_connection: Connection,
    message_tx: broadcast::Sender<String>,
}

impl QuicConnectionManager {
    pub fn new(endpoint: Endpoint) -> Self {
        Self {
            connections: Arc::new(Mutex::new(HashMap::new())),
            endpoint,
        }
    }

    pub async fn add_connection(&self, connection: QuicConnection) {
        let mut connections = self.connections.lock().await;
        connections.insert(connection.connection_id.clone(), connection);
        tracing::info!("QUIC WebSocket connection added: total {}", connections.len());
    }

    pub async fn broadcast_message(&self, message: &str) -> usize {
        let connections = self.connections.lock().await;
        let mut sent_count = 0;
        
        for (connection_id, connection) in connections.iter() {
            // Send message through QUIC stream
            if let Ok(mut send_stream) = connection.quic_connection.open_uni().await {
                if send_stream.write_all(message.as_bytes()).await.is_ok() {
                    let _ = send_stream.finish().await;
                    sent_count += 1;
                } else {
                    tracing::warn!("Failed to send message to connection {}", connection_id);
                }
            }
            
            // Also broadcast via WebSocket for backward compatibility
            if connection.message_tx.send(message.to_string()).is_ok() {
                // Message sent successfully
            }
        }
        
        tracing::info!("Message broadcasted to {} connections", sent_count);
        sent_count
    }

    pub async fn remove_connection(&self, connection_id: &str) {
        let mut connections = self.connections.lock().await;
        if let Some(connection) = connections.remove(connection_id) {
            connection.quic_connection.close(0u32.into(), b"Connection closed");
            tracing::info!("QUIC WebSocket connection removed: remaining {}", connections.len());
        }
    }
}

static CONNECTION_MANAGER: LazyLock<Option<QuicConnectionManager>> = LazyLock::new(|| None);

// QUIC-enabled WebSocket handler
pub async fn quic_websocket_handler(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(handle_quic_websocket_connection)
}

async fn handle_quic_websocket_connection(socket: WebSocket) {
    let connection_id = Uuid::new_v4().to_string();
    tracing::info!("New QUIC WebSocket connection: {}", connection_id);

    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = broadcast::channel::<String>(64);

    // Send connection established message with QUIC information
    let welcome_message = json!({
        "type": "connection_established",
        "connection_id": connection_id,
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "protocol_version": "1.0.0",
        "transport": {
            "type": "quic",
            "version": "1.0",
            "features": ["multiplexing", "connection_migration", "low_latency"]
        }
    });

    if let Err(e) = sender.send(axum::extract::ws::Message::Text(welcome_message.to_string().into())).await {
        tracing::error!("Failed to send welcome message: {}", e);
        return;
    }

    // Message sending task with QUIC optimization
    let sender_task = {
        let connection_id = connection_id.clone();
        tokio::spawn(async move {
            while let Ok(message) = rx.recv().await {
                // Prioritize messages for low-latency delivery
                let message_with_priority = json!({
                    "transport_meta": {
                        "stream_priority": "normal",
                        "delivery_guarantee": "reliable"
                    },
                    "payload": serde_json::from_str::<Value>(&message).unwrap_or(json!(message))
                });
                
                if let Err(e) = sender.send(axum::extract::ws::Message::Text(message_with_priority.to_string().into())).await {
                    tracing::error!("Failed to send message to connection {}: {}", connection_id, e);
                    break;
                }
            }
        })
    };

    // Message receiving task with QUIC stream handling
    let receiver_task = {
        let connection_id = connection_id.clone();
        tokio::spawn(async move {
            while let Some(msg) = receiver.next().await {
                match msg {
                    Ok(axum::extract::ws::Message::Text(text)) => {
                        tracing::info!("Message received from QUIC connection {}: {}", connection_id, text);
                        
                        if let Err(e) = handle_incoming_message(&connection_id, &text).await {
                            tracing::error!("Message processing error: {}", e);
                        }
                    }
                    Ok(axum::extract::ws::Message::Close(_)) => {
                        tracing::info!("QUIC connection {} closed", connection_id);
                        break;
                    }
                    Err(e) => {
                        tracing::error!("QUIC WebSocket error for connection {}: {}", connection_id, e);
                        break;
                    }
                    _ => {}
                }
            }
        })
    };

    // Wait for either task to complete
    tokio::select! {
        _ = sender_task => {
            tracing::info!("QUIC sender task ended: {}", connection_id);
        }
        _ = receiver_task => {
            tracing::info!("QUIC receiver task ended: {}", connection_id);
        }
    }

    // Remove connection from manager
    if let Some(ref manager) = *CONNECTION_MANAGER {
        manager.remove_connection(&connection_id).await;
    }
    tracing::info!("QUIC WebSocket connection closed: {}", connection_id);
}

async fn handle_incoming_message(connection_id: &str, message: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let parsed_message: Value = serde_json::from_str(message)?;
    
    // Extract QUIC-specific metadata if present
    let transport_meta = parsed_message.get("transport_meta");
    let payload = parsed_message.get("payload").unwrap_or(&parsed_message);
    
    match payload.get("type").and_then(|t| t.as_str()) {
        Some("chat_message") => {
            handle_chat_message(connection_id, payload).await?;
        }
        Some("custom_data") => {
            handle_custom_data(connection_id, payload).await?;
        }
        Some("ping") => {
            handle_ping_with_quic_info(connection_id, payload, transport_meta).await?;
        }
        _ => {
            send_error_response(connection_id, "unknown_message_type", "Unknown message type").await?;
        }
    }
    
    Ok(())
}

async fn handle_ping_with_quic_info(connection_id: &str, message: &Value, transport_meta: Option<&Value>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let ping_message = message.get("message").and_then(|v| v.as_str()).unwrap_or("ping");
    let sequence = message.get("sequence").and_then(|v| v.as_u64()).unwrap_or(0);
    let client_timestamp = message.get("client_timestamp").and_then(|v| v.as_str()).unwrap_or("");
    
    let server_timestamp = chrono::Utc::now();
    
    let pong_response = json!({
        "type": "pong",
        "message": format!("pong: {}", ping_message),
        "sequence": sequence,
        "client_timestamp": client_timestamp,
        "server_timestamp": server_timestamp.to_rfc3339(),
        "transport_info": {
            "protocol": "quic",
            "features": ["low_latency", "multiplexing", "connection_migration"],
            "stream_priority": transport_meta.and_then(|m| m.get("stream_priority")).unwrap_or(&json!("normal"))
        },
        "server_info": "Unison Protocol Server with QUIC v1.0.0"
    });
    
    if let Some(ref manager) = *CONNECTION_MANAGER {
        manager.broadcast_message(&pong_response.to_string()).await;
    }
    
    tracing::info!("QUIC Ping response sent: {} -> pong", ping_message);
    
    Ok(())
}
```

### JavaScript/TypeScript Implementation with QUIC

```typescript
interface QuicTransportMeta {
  stream_priority?: 'low' | 'normal' | 'high';
  delivery_guarantee?: 'reliable' | 'unreliable';
  ordered?: boolean;
}

interface MessageWithQuic<T> {
  transport_meta?: QuicTransportMeta;
  payload: T;
}

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

type Message = ChatMessage | CustomData;

class UnisonQuicWebSocketClient {
  private ws: WebSocket | null = null;
  private messageHandlers = new Map<string, (message: any) => void>();
  private quicSupported = false;

  constructor(private url: string) {}

  connect(): Promise<void> {
    return new Promise((resolve, reject) => {
      this.ws = new WebSocket(this.url);
      
      this.ws.onopen = () => {
        console.log('✅ QUIC WebSocket connection established');
        resolve();
      };
      
      this.ws.onerror = (error) => {
        console.error('❌ QUIC WebSocket connection error:', error);
        reject(error);
      };
      
      this.ws.onmessage = (event) => {
        try {
          const message = JSON.parse(event.data);
          
          // Check for QUIC transport information
          if (message.type === 'connection_established' && message.transport?.type === 'quic') {
            this.quicSupported = true;
            console.log('🚀 QUIC transport features:', message.transport.features);
          }
          
          this.handleMessage(message);
        } catch (error) {
          console.error('Message parsing error:', error);
        }
      };
      
      this.ws.onclose = () => {
        console.log('📴 QUIC WebSocket connection closed');
      };
    });
  }

  private handleMessage(message: any) {
    // Handle messages with QUIC metadata
    const payload = message.payload || message;
    const transportMeta = message.transport_meta;
    
    if (transportMeta) {
      console.log('📦 QUIC transport metadata:', transportMeta);
    }
    
    const handler = this.messageHandlers.get(payload.type);
    if (handler) {
      handler(payload);
    } else {
      console.warn('Unhandled message type:', payload.type);
    }
  }

  onMessage<T extends Message>(type: T['type'], handler: (message: T) => void) {
    this.messageHandlers.set(type, handler);
  }

  sendMessage(message: Message, transportMeta?: QuicTransportMeta) {
    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      const messageWithMeta: MessageWithQuic<Message> = {
        payload: message
      };
      
      // Add QUIC transport metadata if supported
      if (this.quicSupported && transportMeta) {
        messageWithMeta.transport_meta = transportMeta;
      }
      
      this.ws.send(JSON.stringify(messageWithMeta));
    } else {
      console.error('WebSocket connection not available');
    }
  }

  sendChatMessage(userName: string, content: string, room = 'general', priority: 'low' | 'normal' | 'high' = 'normal') {
    const message: ChatMessage = {
      type: 'chat_message',
      user_name: userName,
      content,
      room
    };
    
    // Use high priority for urgent messages
    const transportMeta: QuicTransportMeta = {
      stream_priority: priority,
      delivery_guarantee: 'reliable',
      ordered: true
    };
    
    this.sendMessage(message, transportMeta);
  }

  sendCustomData(dataType: string, payload: any, sender?: string, priority: 'low' | 'normal' | 'high' = 'normal') {
    const message: CustomData = {
      type: 'custom_data',
      data_type: dataType,
      payload,
      sender
    };
    
    const transportMeta: QuicTransportMeta = {
      stream_priority: priority,
      delivery_guarantee: 'reliable',
      ordered: false // Allow out-of-order delivery for better performance
    };
    
    this.sendMessage(message, transportMeta);
  }

  // Send ping with QUIC performance measurement
  sendPing(sequence: number = 1): Promise<number> {
    const startTime = performance.now();
    
    return new Promise((resolve) => {
      const pingMessage = {
        type: 'ping',
        message: 'quic_latency_test',
        sequence,
        client_timestamp: new Date().toISOString()
      };
      
      const handler = (message: any) => {
        if (message.type === 'pong' && message.sequence === sequence) {
          const endTime = performance.now();
          const latency = endTime - startTime;
          
          console.log(`🏓 QUIC Ping latency: ${latency.toFixed(2)}ms`);
          console.log('🚀 Transport features:', message.transport_info?.features);
          
          this.messageHandlers.delete('pong');
          resolve(latency);
        }
      };
      
      this.onMessage('pong', handler);
      
      // Send with high priority for accurate latency measurement
      this.sendMessage(pingMessage, {
        stream_priority: 'high',
        delivery_guarantee: 'reliable',
        ordered: true
      });
    });
  }

  disconnect() {
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
  }
}

// Usage example with QUIC features
async function example() {
  const client = new UnisonQuicWebSocketClient('ws://localhost:8080/ws');
  
  // Set up message handlers
  client.onMessage('chat_message', (message) => {
    console.log(`💬 ${message.user_name}: ${message.content}`);
  });
  
  client.onMessage('system_notification', (message) => {
    console.log(`🔔 ${message.title}: ${message.message}`);
  });
  
  // Connect and test
  await client.connect();
  
  // Send high-priority urgent message
  client.sendChatMessage('Developer A', 'Urgent: System maintenance starting!', 'alerts', 'high');
  
  // Send custom data with normal priority
  client.sendCustomData('progress_update', {
    task_id: 'task_001',
    progress: 0.75,
    status: 'processing'
  }, 'task_service', 'normal');
  
  // Measure QUIC latency
  const latency = await client.sendPing();
  console.log(`📊 QUIC connection latency: ${latency}ms`);
}
```

## QUIC Benefits for Real-time Messaging

### 1. Reduced Latency
- **0-RTT Connection**: Resume connections without handshake
- **Multiplexing**: Multiple streams without blocking
- **Connection Migration**: Seamless network switching

### 2. Enhanced Performance
- **Advanced Congestion Control**: Better throughput optimization
- **Stream Prioritization**: Critical messages get priority
- **Head-of-line Blocking Elimination**: Independent stream processing

### 3. Built-in Security
- **TLS 1.3 by Default**: Always encrypted communication
- **Connection ID**: Protection against connection hijacking
- **Forward Secrecy**: Past communications remain secure

## Best Practices with QUIC

### 1. Stream Prioritization
```rust
// High priority for critical system messages
let high_priority_stream = connection.open_uni_with_priority(Priority::High).await?;

// Normal priority for chat messages
let normal_priority_stream = connection.open_uni().await?;

// Low priority for background data
let low_priority_stream = connection.open_uni_with_priority(Priority::Low).await?;
```

### 2. Connection Migration Handling
```javascript
// Handle network changes gracefully
client.onConnectionMigration((oldEndpoint, newEndpoint) => {
    console.log(`🔄 Connection migrated from ${oldEndpoint} to ${newEndpoint}`);
    // Automatically resume message flow
});
```

### 3. Error Recovery
```typescript
interface QuicError {
  error_code: number;
  frame_type?: number;
  reason?: string;
}

client.onQuicError((error: QuicError) => {
  console.error('QUIC transport error:', error);
  // Implement automatic reconnection logic
  if (error.error_code === QUIC_ERROR_CODES.CONNECTION_TIMEOUT) {
    client.reconnect();
  }
});
```

## Performance Optimization

### 1. Connection Pooling
```rust
pub struct QuicConnectionPool {
    connections: Vec<Connection>,
    max_connections: usize,
}

impl QuicConnectionPool {
    pub fn get_connection(&mut self) -> Option<&Connection> {
        self.connections.iter().find(|conn| !conn.is_closed())
    }
    
    pub async fn create_connection(&mut self, endpoint: &str) -> Result<Connection> {
        // Implement connection creation with 0-RTT if possible
        let connection = self.endpoint.connect(endpoint, "unison-protocol")?
            .into_0rtt()
            .map_err(|_| "0-RTT not available")?
            .0;
        
        self.connections.push(connection.clone());
        Ok(connection)
    }
}
```

### 2. Message Batching
```typescript
class MessageBatcher {
  private batch: Message[] = [];
  private batchTimeout: NodeJS.Timeout | null = null;
  
  addMessage(message: Message) {
    this.batch.push(message);
    
    if (this.batch.length >= 10) {
      this.flushBatch();
    } else if (!this.batchTimeout) {
      this.batchTimeout = setTimeout(() => this.flushBatch(), 100);
    }
  }
  
  private flushBatch() {
    if (this.batch.length > 0) {
      // Send batched messages over single QUIC stream
      this.client.sendBatch(this.batch);
      this.batch = [];
    }
    
    if (this.batchTimeout) {
      clearTimeout(this.batchTimeout);
      this.batchTimeout = null;
    }
  }
}
```

## Testing and Debugging

### QUIC Connection Testing
```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>Unison Protocol QUIC WebSocket Test</title>
</head>
<body>
    <div id="quic-status">Checking QUIC support...</div>
    <div id="connection-info"></div>
    <div id="messages"></div>
    
    <script>
        async function testQuicConnection() {
            const client = new UnisonQuicWebSocketClient('ws://localhost:8080/ws');
            
            try {
                await client.connect();
                
                // Test latency
                const latencies = [];
                for (let i = 0; i < 10; i++) {
                    const latency = await client.sendPing(i);
                    latencies.push(latency);
                    await new Promise(resolve => setTimeout(resolve, 100));
                }
                
                const avgLatency = latencies.reduce((a, b) => a + b) / latencies.length;
                document.getElementById('connection-info').innerHTML = 
                    `<p>Average QUIC latency: ${avgLatency.toFixed(2)}ms</p>
                     <p>Min: ${Math.min(...latencies).toFixed(2)}ms</p>
                     <p>Max: ${Math.max(...latencies).toFixed(2)}ms</p>`;
                
            } catch (error) {
                document.getElementById('quic-status').innerHTML = 
                    `<span style="color: red;">QUIC connection failed: ${error}</span>`;
            }
        }
        
        testQuicConnection();
    </script>
</body>
</html>
```

## References

- [QUIC Protocol RFC 9000](https://www.rfc-editor.org/rfc/rfc9000.html)
- [HTTP/3 over QUIC RFC 9114](https://www.rfc-editor.org/rfc/rfc9114.html)
- [KDL (KDL Document Language) Specification](https://kdl.dev/)
- [WebSocket over QUIC Transport](https://datatracker.ietf.org/doc/draft-ietf-masque-h3-websockets/)

---

**Last Updated**: January 2024 | **Version**: 1.0.0 with QUIC support