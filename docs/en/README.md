# Unison Protocol Documentation

**English** | [日本語](./README.md)

## Overview

Unison Protocol is a type-safe messaging specification based on KDL (KDL Document Language). It enables high-performance and reliable real-time communication through adaptive transport (QUIC-preferred with WebSocket fallback).

## Documentation Index

### Messaging Systems
- **[WebSocket Messaging (English)](./websocket-messaging-en.md)** - Implementation guide for WebSocket messaging with Unison Protocol
- **[WebSocketメッセージング（日本語）](./websocket-messaging-ja.md)** - Unison Protocolを使用したWebSocketメッセージングの実装ガイド

### Key Features

#### 🔒 Type Safety
- Strict type definitions with KDL schemas
- Compile-time and runtime type checking
- Automatic validation capabilities

#### 🌐 Adaptive Transport
- **QUIC Preferred**: Automatically selected when high-performance QUIC is available
- **WebSocket Fallback**: Automatic fallback to WebSocket in unsupported environments
- **Transparent API**: Same API regardless of transport layer

#### 🚀 Performance Optimization
- Low-latency communication
- Multi-stream support
- Efficient message broadcasting
- Connection migration support

## Quick Start

### Basic Usage Example

```typescript
import { UnisonAdaptiveClient } from 'unison-protocol';

const client = new UnisonAdaptiveClient('ws://localhost:8080');

// Set up message handlers
client.onMessage('chat_message', (message) => {
    console.log(`💬 ${message.user_name}: ${message.content}`);
});

// Connect (automatically selects QUIC or WebSocket)
await client.connect();

// Send message
client.sendChatMessage('Developer', 'Hello, World!');
```

### KDL Schema Example

```kdl
protocol "my-app" version="1.0.0" {
    namespace "com.example.messaging"
    
    transport "adaptive" {
        primary "quic" {
            detection_timeout_ms 5000
        }
        fallback "websocket" {
            subprotocol "unison-messaging-v1"
        }
    }
    
    message "ChatMessage" {
        field "user_name" type="string" required=true
        field "content" type="string" required=true
        field "timestamp" type="timestamp" required=true
    }
}
```

## Implementation Examples

### Supported Languages/Frameworks

- **Rust**: Axum + Quinn (QUIC) / Tokio-Tungstenite (WebSocket)
- **JavaScript/TypeScript**: WebTransport API (QUIC) / WebSocket API
- **Python**: asyncio + aioquic (QUIC) / websockets (WebSocket)

### Sample Applications

Implementation examples are included in each document:

1. **Chat Application** - Real-time message exchange
2. **Data Streaming** - Efficient transfer of large datasets
3. **System Monitoring** - Performance metrics distribution
4. **Game Communication** - Low-latency game data communication

## Architecture

```
┌─────────────────────────────────────────┐
│        Application Layer                │
├─────────────────────────────────────────┤
│        Unison Protocol Layer            │
│  ┌─────────────┐ ┌─────────────────────┐│
│  │ KDL Schema  │ │ Message Validation  ││
│  └─────────────┘ └─────────────────────┘│
├─────────────────────────────────────────┤
│        Adaptive Transport Layer         │
│  ┌─────────────┐ ┌─────────────────────┐│
│  │    QUIC     │ │     WebSocket       ││
│  │ (Preferred) │ │    (Fallback)       ││
│  └─────────────┘ └─────────────────────┘│
├─────────────────────────────────────────┤
│           Network Layer                 │
└─────────────────────────────────────────┘
```

## Performance Comparison

| Feature | QUIC | WebSocket | Improvement |
|---------|------|-----------|-------------|
| Connection Time | ~50ms | ~150ms | **66% faster** |
| Latency | ~20ms | ~35ms | **43% lower** |
| Throughput | ~1.2Gbps | ~800Mbps | **50% higher** |
| CPU Usage | Low | Medium | **More efficient** |

## Security

### Standard Security Features
- **TLS 1.3**: Latest encryption protocol
- **Forward Secrecy**: Protection of past communications
- **Connection ID**: Protection against connection hijacking
- **Automatic Certificate Validation**: Prevention of man-in-the-middle attacks

### Implementation Recommendations
- Regular connection re-establishment
- Message integrity checks
- Rate limiting implementation
- Log auditing capabilities

## Troubleshooting

### Common Issues

1. **QUIC Connection Failure**
   - Check firewall settings
   - Verify UDP port 443 is open
   - Ensure WebSocket fallback works correctly

2. **Performance Issues**
   - Check network bandwidth
   - Monitor CPU usage
   - Adjust message batching settings

3. **Compatibility Issues**
   - Check browser WebTransport API support
   - Verify KDL schema version compatibility

### Debugging Tools

- **Connection Diagnostics**: Transport layer health checks
- **Message Tracing**: Message flow visualization
- **Performance Metrics**: Latency and throughput monitoring

## Development and Contributing

### Development Environment Setup

```bash
# Clone repository
git clone https://github.com/example/unison-protocol.git

# Install dependencies
cd unison-protocol
cargo build  # Rust
npm install  # Node.js
pip install -r requirements.txt  # Python
```

### Running Tests

```bash
# Run all tests
cargo test        # Rust
npm test         # Node.js
pytest          # Python

# Integration tests
./scripts/test-integration.sh
```

## License and Usage

Unison Protocol is distributed under the [MIT License](../LICENSE).

### Commercial Use
- Unlimited commercial use allowed
- Attribution required
- No warranty provided

## Support and Community

- **Issues**: [GitHub Issues](https://github.com/example/unison-protocol/issues)
- **Discussions**: [GitHub Discussions](https://github.com/example/unison-protocol/discussions)
- **Documentation**: [Wiki](https://github.com/example/unison-protocol/wiki)

---

**Last Updated**: January 2024 | **Version**: 1.0.0