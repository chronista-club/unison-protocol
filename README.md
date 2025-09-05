# 🎵 Unison Protocol

*Harmonizing communication across languages and platforms*

[![Crates.io](https://img.shields.io/crates/v/unison-protocol.svg)](https://crates.io/crates/unison-protocol)
[![Documentation](https://docs.rs/unison-protocol/badge.svg)](https://docs.rs/unison-protocol)
[![Build Status](https://github.com/chronista-club/unison-protocol/workflows/CI/badge.svg)](https://github.com/chronista-club/unison-protocol/actions)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

**Unison Protocol** is a modern KDL-based type-safe communication framework built on QUIC transport. It provides seamless client-server communication with automatic code generation, ultra-low latency, and comprehensive schema validation.

## ✨ Features

- **🎯 Type-safe communication**: Automatic code generation from KDL protocol definitions
- **⚡ QUIC transport**: Ultra-low latency communication with TLS 1.3 encryption
- **🔐 Built-in security**: TLS 1.3, certificate management with rust-embed support
- **🔍 Schema validation**: Compile-time and runtime protocol validation
- **🚀 Async-first**: Built with tokio and async/await from the ground up
- **📚 Rich protocol definitions**: Support for services, messages, methods, and complex types
- **🌊 Bidirectional streaming**: SystemStream trait for full-duplex QUIC communication
- **🎵 Service-oriented architecture**: High-level Service trait with metadata and lifecycle management
- **🔧 Developer-friendly**: Simple API with comprehensive error handling and logging
- **🧪 Comprehensive testing**: Integrated tests with single-process client-server testing

## 🚀 Quick Start

### 1. Define Your Protocol

Create a KDL schema file (e.g., `my_protocol.kdl`):

```kdl
protocol "my-service" version="1.0.0" {
    namespace "my.service"
    description "My awesome service protocol"
    
    service "UserService" {
        method "create_user" {
            description "Create a new user"
            request {
                field "name" type="string" required=true
                field "age" type="number" required=false
            }
            response {
                field "user_id" type="string" required=true
                field "message" type="string" required=true
            }
        }
    }
}
```

### 2. Server Implementation

```rust
use anyhow::Result;
use unison_protocol::{UnisonProtocol, UnisonServer, UnisonServerExt};
use unison_protocol::network::NetworkError;
use serde_json::json;
use tracing::{info, Level};
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("🎵 Unison Protocol Server Starting...");
    
    // Create protocol instance and load schema
    let mut protocol = UnisonProtocol::new();
    protocol.load_schema(include_str!("../schemas/ping_pong.kdl"))?;
    
    let mut server = protocol.create_server();
    let start_time = Instant::now();
    
    // Register handlers
    server.register_handler("ping", move |payload| {
        let message = payload.get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("Hello from client!")
            .to_string();
            
        let sequence = payload.get("sequence")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        
        info!("🎵 Received ping: \"{}\" (seq: {})", message, sequence);
        
        let response = json!({
            "message": format!("Pong: {}", message),
            "sequence": sequence,
            "server_info": "Unison Protocol Server v1.0.0"
        });
        
        Ok(response) as Result<serde_json::Value, NetworkError>
    });
    
    info!("📡 Listening on: quic://127.0.0.1:8080 (QUIC Transport)");
    server.listen("127.0.0.1:8080").await?;
    
    Ok(())
}
```

### 3. Client Implementation

```rust
use anyhow::Result;
use unison_protocol::{UnisonProtocol, UnisonClient};
use serde_json::json;
use tracing::{info, Level};
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("🔌 Unison Protocol Client Starting...");
    
    // Create protocol instance and load schema
    let mut protocol = UnisonProtocol::new();
    protocol.load_schema(include_str!("../schemas/ping_pong.kdl"))?;
    
    let mut client = protocol.create_client();
    
    info!("🔌 Connecting to quic://127.0.0.1:8080");
    client.connect("127.0.0.1:8080").await?;
    info!("✅ Connected to Unison Protocol server");
    
    // Send ping request
    let start_time = Instant::now();
    let response = client.call("ping", json!({
        "message": "Hello from Rust client!",
        "sequence": 1
    })).await?;
    let elapsed = start_time.elapsed();
    
    info!("📨 Received response in {:?}", elapsed);
    info!("📋 Server response: {}", serde_json::to_string_pretty(&response)?);
    
    client.disconnect().await?;
    info!("👋 Disconnected from server");
    
    Ok(())
}
```

## 📂 Project Structure

```
unison-protocol/
├── src/
│   ├── lib.rs              # Main library interface
│   ├── core/               # Core protocol types
│   │   └── mod.rs          # UnisonMessage, UnisonResponse, etc.
│   ├── parser/             # KDL schema parser
│   │   ├── mod.rs          # Parser interface
│   │   ├── schema.rs       # Schema parsing logic
│   │   └── types.rs        # Type definitions
│   ├── codegen/            # Code generators
│   │   ├── mod.rs          # Generator interface
│   │   ├── rust.rs         # Rust code generation
│   │   └── typescript.rs   # TypeScript code generation
│   └── network/            # Network implementation
│       ├── mod.rs          # Network traits, SystemStream, and errors
│       ├── client.rs       # Protocol client implementation
│       ├── server.rs       # Protocol server implementation
│       ├── service.rs      # Service trait and implementations
│       └── quic.rs         # QUIC transport with rustls/quinn
├── schemas/                # Protocol definitions
│   ├── unison_core.kdl     # Core Unison protocol
│   ├── ping_pong.kdl       # Example ping-pong protocol
│   └── diarkis_devtools.kdl # Diarkis DevTools protocol
├── assets/                 # Static assets
│   └── certs/              # Auto-generated QUIC certificates
│       ├── cert.pem        # Server certificate
│       └── private_key.der # Private key
├── examples/               # Usage examples
│   ├── unison_ping_server.rs # QUIC server example
│   └── unison_ping_client.rs # QUIC client example
├── tests/                  # Integration tests
│   ├── simple_quic_test.rs      # QUIC functionality tests
│   ├── quic_integration_test.rs # Full client-server integration
│   └── system_stream_test.rs    # SystemStream and Service tests
├── build.rs                # Build script for code generation
├── Cargo.toml              # Rust dependencies
└── README.md               # This file
```

## 🔧 Examples

The repository includes comprehensive examples demonstrating QUIC-based communication:

### Run the Ping-Pong Server
```bash
# Start the QUIC server (auto-generates certificates)
cargo run --example unison_ping_server
```

Output:
```
🎵 Unison Protocol Ping Server Starting...
📡 Listening on: quic://127.0.0.1:8080 (QUIC Transport)
📊 Available methods: ping, echo, get_server_time
⏹️  Press Ctrl+C to stop
```

### Run the Ping-Pong Client
```bash
# In another terminal
cargo run --example unison_ping_client
```

Output:
```
🔌 Unison Protocol Client Starting...
✅ Connected to Unison Protocol server
📨 Received response in 23.5ms
📋 Server response: {
  "message": "Pong: Hello from Rust client!",
  "sequence": 1,
  "server_info": "Unison Protocol Server v1.0.0"
}
```

### Available Protocol Methods

The ping-pong example demonstrates these Unison Protocol methods:

- **`ping`**: Basic request-response with message echo and sequence tracking
- **`echo`**: Echo any JSON data with optional transformations (uppercase, reverse)  
- **`get_server_time`**: Get server timestamp, timezone, and uptime information

## 🌊 SystemStream & Service Architecture

Unison Protocol provides two levels of abstraction for QUIC communication:

### SystemStream - Low-level Bidirectional Streaming

The `SystemStream` trait provides low-level access to QUIC bidirectional streams:

```rust
use unison_protocol::network::{SystemStream, NetworkError};

#[async_trait]
pub trait SystemStream: Send + Sync {
    async fn send(&mut self, data: serde_json::Value) -> Result<(), NetworkError>;
    async fn receive(&mut self) -> Result<serde_json::Value, NetworkError>;
    fn is_active(&self) -> bool;
    async fn close(&mut self) -> Result<(), NetworkError>;
    fn get_handle(&self) -> StreamHandle;
}
```

### Service - High-level Service Interface

The `Service` trait builds on `SystemStream` for service-oriented communication:

```rust  
use unison_protocol::network::{Service, ServiceConfig, ServicePriority};

#[async_trait]
pub trait Service: SystemStream {
    fn service_type(&self) -> &str;
    fn service_name(&self) -> &str;
    fn metadata(&self) -> HashMap<String, String>;
    fn version(&self) -> &str;
    
    // Service lifecycle
    async fn start_service_heartbeat(&mut self, interval_secs: u64) -> Result<(), NetworkError>;
    async fn service_ping(&mut self) -> Result<(), NetworkError>;
    async fn handle_request(&mut self, method: &str, payload: serde_json::Value) -> Result<serde_json::Value, NetworkError>;
    async fn shutdown(&mut self) -> Result<(), NetworkError>;
}
```

### Service Features

- **🔄 Heartbeat system**: Automatic health monitoring  
- **📊 Performance metrics**: Latency, throughput, error tracking
- **🎯 Service discovery**: Metadata and capability advertising
- **⚡ Real-time operations**: Priority-based message handling
- **🔧 Lifecycle management**: Graceful startup and shutdown

## 🧪 Testing

Unison Protocol includes comprehensive testing for QUIC functionality:

### Run All Tests
```bash
# Run all tests including integration tests
cargo test

# Run with detailed logging
RUST_LOG=info cargo test -- --nocapture
```

### Test Categories

#### Unit Tests
```bash
# Test individual components
cargo test --lib
```

#### QUIC Integration Tests
```bash
# Test QUIC client-server communication
cargo test --test simple_quic_test

# Test full integration (server + client in single process)
cargo test --test quic_integration_test

# Test SystemStream and Service functionality
cargo test --test system_stream_test
```

### Test Coverage

- **QUIC Configuration**: Server and client transport setup
- **Certificate Management**: Auto-generation and rust-embed support
- **Handler Registration**: Method routing and error handling
- **Message Serialization**: JSON payload handling
- **Performance Testing**: Latency and throughput validation
- **Error Scenarios**: Connection failures and timeouts
- **SystemStream Testing**: Bidirectional streaming functionality
- **Service Testing**: Service lifecycle, heartbeats, and metadata management

## 🔌 Integration

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
unison-protocol = "0.1.0"

# Required runtime dependencies
tokio = { version = "1.40", features = ["full"] }
serde_json = "1.0"
anyhow = "1.0"  # For error handling
tracing = "0.1"  # For logging
tracing-subscriber = { version = "0.3", features = ["fmt"] }

# Optional: for time handling in examples
chrono = { version = "0.4", features = ["serde"] }
```

Or install via cargo:

```bash
cargo add unison-protocol
cargo add tokio --features full
cargo add serde_json anyhow tracing tracing-subscriber
```

### Prerequisites

- **Rust**: 1.70+ (2021 edition)
- **QUIC Support**: Automatic certificate generation via rcgen
- **Platform**: Cross-platform (Windows, macOS, Linux)

### Transport and Security

#### QUIC Transport Features
- ✅ **TLS 1.3**: Modern encryption by default
- ✅ **0-RTT**: Ultra-low latency connection establishment
- ✅ **Multiplexing**: Multiple streams over single connection
- ✅ **Connection Migration**: Robust connectivity
- ✅ **Auto Certificate Generation**: Development-ready out of the box

#### Language Support Roadmap
- ✅ **Rust**: Full async/await support with quinn/rustls
- 🚧 **TypeScript**: Code generation (in development)
- 📅 **Python**: Planned (aioquic integration)
- 📅 **Go**: Planned (quic-go integration)
- 📅 **JavaScript (Node.js)**: Planned (WebTransport API)

## 🤝 Contributing

We welcome contributions! Please see our contributing guidelines:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📄 License

This project is licensed under the [MIT License](LICENSE).

---

**Unison Protocol** - *Harmonizing communication across languages and platforms* 🎵