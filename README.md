# 🎵 Unison Protocol

*Next-generation type-safe communication protocol framework*

[![Crates.io](https://img.shields.io/crates/v/unison-protocol.svg)](https://crates.io/crates/unison-protocol)
[![Documentation](https://docs.rs/unison-protocol/badge.svg)](https://docs.rs/unison-protocol)
[![Build Status](https://github.com/chronista-club/unison-protocol/workflows/CI/badge.svg)](https://github.com/chronista-club/unison-protocol/actions)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

[English](README.md) | [日本語](docs/ja/README.md)

## 📌 Overview

**Unison Protocol** is a type-safe communication protocol framework based on KDL (KDL Document Language). Leveraging QUIC transport, it supports building fast, secure, and extensible distributed systems.

### 🎯 Key Features

- **Type-safe Communication**: Automatic code generation from KDL schemas
- **Ultra-low Latency**: High-speed communication via QUIC (HTTP/3) transport
- **Built-in Security**: TLS 1.3 encryption with automatic development certificate generation
- **CGP (Context-Generic Programming) Support**: Extensible handler system
- **Async-first**: Fully asynchronous implementation based on Tokio
- **Bidirectional Streaming**: Full-duplex communication via UnisonStream
- **Service-oriented**: Lifecycle management via high-level Service trait

## 🚀 Quick Start

### Installation

```toml
[dependencies]
unison-protocol = "0.1.0-alpha1"
tokio = { version = "1.40", features = ["full"] }
serde_json = "1.0"
anyhow = "1.0"
tracing = "0.1"
```

### Basic Usage

#### 1. Protocol Definition (KDL)

```kdl
// schemas/my_service.kdl
protocol "my-service" version="1.0.0" {
    namespace "com.example.myservice"

    service "UserService" {
        method "createUser" {
            request {
                field "name" type="string" required=true
                field "email" type="string" required=true
            }
            response {
                field "id" type="string" required=true
                field "created_at" type="timestamp" required=true
            }
        }
    }
}
```

#### 2. Server Implementation

```rust
use unison_protocol::{ProtocolServer, NetworkError};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut server = ProtocolServer::new();

    // Register handler
    server.register_handler("createUser", |payload| {
        let name = payload["name"].as_str().unwrap();
        let email = payload["email"].as_str().unwrap();

        // User creation logic
        Ok(json!({
            "id": uuid::Uuid::new_v4().to_string(),
            "created_at": chrono::Utc::now().to_rfc3339()
        }))
    });

    // Start QUIC server
    server.listen("127.0.0.1:8080").await?;
    Ok(())
}
```

#### 3. Client Implementation

```rust
use unison_protocol::ProtocolClient;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = ProtocolClient::new();

    // Connect to server
    client.connect("127.0.0.1:8080").await?;

    // RPC call
    let response = client.call("createUser", json!({
        "name": "Alice",
        "email": "alice@example.com"
    })).await?;

    println!("Created user: {}", response);
    Ok(())
}
```

## 🏗️ Architecture

### Component Structure

```
unison-protocol/
├── 🎯 Core Layer
│   ├── parser/          # KDL schema parser
│   ├── codegen/        # Code generators (Rust/TypeScript)
│   └── types/          # Basic type definitions
│
├── 🌐 Network Layer
│   ├── quic/           # QUIC transport implementation
│   ├── client/         # Protocol client
│   ├── server/         # Protocol server
│   └── service/        # Service abstraction layer
│
└── 🧩 Context Layer (CGP)
    ├── adapter/        # Existing system integration
    ├── handlers/       # Extensible handlers
    └── traits/         # Generic trait definitions
```

### Core Components

#### 1. **UnisonStream** - Low-level Bidirectional Streaming

```rust
pub trait UnisonStream: Send + Sync {
    async fn send(&mut self, data: Value) -> Result<(), NetworkError>;
    async fn receive(&mut self) -> Result<Value, NetworkError>;
    async fn close(&mut self) -> Result<(), NetworkError>;
    fn is_active(&self) -> bool;
}
```

#### 2. **Service** - High-level Service Abstraction

```rust
pub trait Service: UnisonStream {
    fn service_type(&self) -> &str;
    fn version(&self) -> &str;
    async fn handle_request(&mut self, method: &str, payload: Value)
        -> Result<Value, NetworkError>;
}
```

#### 3. **CGP Context** - Extensible Context

```rust
pub struct CgpProtocolContext<T, R, H> {
    transport: T,      // Transport layer
    registry: R,       // Service registry
    handlers: H,       // Message handlers
}
```

## 📊 Performance

### Benchmark Results

| Metric | QUIC | WebSocket | HTTP/2 |
|--------|------|-----------|--------|
| Latency (p50) | 2.3ms | 5.1ms | 8.2ms |
| Latency (p99) | 12.5ms | 23.4ms | 45.6ms |
| Throughput | 850K msg/s | 420K msg/s | 180K msg/s |
| CPU Usage | 35% | 48% | 62% |

*Test environment: AMD Ryzen 9 5900X, 32GB RAM, localhost*

## 🧪 Testing

### Running Tests

```bash
# Run all tests
cargo test

# Integration tests only
cargo test --test quic_integration_test

# With verbose logging
RUST_LOG=debug cargo test -- --nocapture
```

### Test Coverage

- ✅ QUIC connection/disconnection
- ✅ Message serialization
- ✅ Handler registration/invocation
- ✅ Error handling
- ✅ SystemStream lifecycle
- ✅ Service metadata management
- ✅ Automatic certificate generation

## 🔧 Advanced Usage

### Custom Handler Implementation

```rust
use unison_protocol::context::{Handler, HandlerRegistry};

struct MyCustomHandler;

#[async_trait]
impl Handler for MyCustomHandler {
    async fn handle(&self, input: Value) -> Result<Value, NetworkError> {
        // Custom logic
        Ok(json!({"status": "processed"}))
    }
}

// Registration
let registry = HandlerRegistry::new();
registry.register("custom", MyCustomHandler).await;
```

### Streaming Communication

```rust
use unison_protocol::network::UnisonStream;

// Create stream
let mut stream = client.start_system_stream("data_feed", json!({})).await?;

// Async send/receive
tokio::spawn(async move {
    while stream.is_active() {
        match stream.receive().await {
            Ok(data) => println!("Received: {}", data),
            Err(e) => eprintln!("Error: {}", e),
        }
    }
});
```

### Service Metrics

```rust
let stats = service.get_performance_stats().await?;
println!("Latency: {:?}", stats.avg_latency);
println!("Throughput: {} msg/s", stats.messages_per_second);
println!("Active streams: {}", stats.active_streams);
```

## 📚 Documentation

- [API Reference](https://docs.rs/unison-protocol)
- [Protocol Specification](docs/en/PROTOCOL_SPEC.md)
- [Architecture Guide](docs/en/ARCHITECTURE.md)
- [Contribution Guide](CONTRIBUTING.md)

## 🛠️ Development

### Build Requirements

- Rust 1.70 or higher
- Tokio 1.40 or higher
- OpenSSL or BoringSSL (for QUIC)

### Development Environment Setup

```bash
# Clone repository
git clone https://github.com/chronista-club/unison-protocol
cd unison-protocol

# Install dependencies
cargo build

# Start development server
cargo run --example unison_ping_server

# Run tests
cargo test
```

### Code Generation

```bash
# Generate code from KDL schema
cargo build --features codegen

# Generate TypeScript definitions
cargo run --bin generate-ts
```

## 🤝 Contributing

Pull requests are welcome! Please follow these guidelines:

1. Fork and create a feature branch
2. Add tests (coverage 80% or higher)
3. Run `cargo fmt` and `cargo clippy`
4. Use [Conventional Commits](https://www.conventionalcommits.org/) for commit messages
5. Submit a pull request

## 📄 License

MIT License - See [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [Quinn](https://github.com/quinn-rs/quinn) - QUIC implementation
- [KDL](https://kdl.dev/) - Configuration language
- [Tokio](https://tokio.rs/) - Async runtime

---

**Unison Protocol** - *Harmonizing communication across languages and platforms* 🎵

[GitHub](https://github.com/chronista-club/unison-protocol) | [Crates.io](https://crates.io/crates/unison-protocol) | [Discord](https://discord.gg/unison-protocol)