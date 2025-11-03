# unison-network

Network layer implementation for Unison Protocol - QUIC transport and peer-to-peer networking.

## Overview

`unison-network` provides the networking foundation for the Unison Protocol framework. It implements:

- **QUIC Transport**: High-performance, low-latency transport layer using Quinn
- **Peer-to-Peer Networking**: Node-to-node communication primitives
- **Connection Management**: Automatic reconnection, keepalive, and health monitoring
- **Stream Multiplexing**: Multiple concurrent streams over a single connection

## Features

- 🚀 Ultra-low latency QUIC transport
- 🔒 TLS 1.3 encryption built-in
- 🌐 IPv6-first design
- 🔄 Automatic connection recovery
- 📊 Network metrics and monitoring
- 🧩 Extensible transport layer

## Usage

```rust
use unison_network::{Node, NodeConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = NodeConfig::default();
    let node = Node::new(config).await?;
    
    // Start listening
    node.listen("[::1]:8080").await?;
    
    Ok(())
}
```

## License

MIT License - see [LICENSE](../../LICENSE) for details.
