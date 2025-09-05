# 🎵 Unison Protocol

*Harmonizing communication across languages and platforms*

[![Crates.io](https://img.shields.io/crates/v/unison-protocol.svg)](https://crates.io/crates/unison-protocol)
[![Documentation](https://docs.rs/unison-protocol/badge.svg)](https://docs.rs/unison-protocol)
[![Build Status](https://github.com/chronista-club/unison-protocol/workflows/CI/badge.svg)](https://github.com/chronista-club/unison-protocol/actions)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

**Unison Protocol** is a KDL-based type-safe communication framework that enables seamless client-server communication with automatic code generation for multiple languages.

## ✨ Features

- **🎯 Type-safe communication**: Automatic code generation from KDL protocol definitions
- **🌐 Multi-language support**: Generate client/server code for Rust, TypeScript, and more
- **⚡ WebSocket-based**: Real-time bidirectional communication
- **🔍 Schema validation**: Compile-time and runtime protocol validation
- **🚀 Async-first**: Built with async/await support from the ground up
- **📚 Rich protocol definitions**: Support for services, messages, methods, and complex types
- **🔧 Developer-friendly**: Simple API with comprehensive error handling

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
use unison_protocol::{UnisonProtocol, UnisonServer};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut protocol = UnisonProtocol::new();
    protocol.load_schema(include_str!("my_protocol.kdl"))?;
    
    let mut server = protocol.create_server();
    server.register_handler("create_user", |payload| {
        let name = payload["name"].as_str().unwrap_or("Anonymous");
        let user_id = uuid::Uuid::new_v4().to_string();
        
        Ok(json!({
            "user_id": user_id,
            "message": format!("Welcome, {}!", name)
        }))
    });
    
    println!("🎵 Unison Protocol server listening on ws://127.0.0.1:8080");
    server.listen("127.0.0.1:8080").await?;
    Ok(())
}
```

### 3. Client Implementation

```rust
use unison_protocol::{UnisonProtocol, UnisonClient};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut protocol = UnisonProtocol::new();
    protocol.load_schema(include_str!("my_protocol.kdl"))?;
    
    let mut client = protocol.create_client();
    client.connect("ws://127.0.0.1:8080").await?;
    
    let response = client.call("create_user", json!({
        "name": "Alice",
        "age": 30
    })).await?;
    
    println!("Response: {}", response);
    client.disconnect().await?;
    Ok(())
}
```

## 📂 Project Structure

```
unison/
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
│       ├── mod.rs          # Network traits and errors
│       ├── client.rs       # Client implementation
│       ├── server.rs       # Server implementation
│       └── websocket.rs    # WebSocket transport
├── schemas/                # Protocol definitions
│   ├── unison_core.kdl     # Core Unison protocol
│   ├── ping_pong.kdl       # Example ping-pong protocol
│   └── diarkis_devtools.kdl # Diarkis DevTools protocol
├── examples/               # Usage examples
│   ├── unison_ping_server.rs # Unison server example
│   └── unison_ping_client.rs # Unison client example
├── Cargo.toml              # Rust dependencies
└── README.md               # This file
```

## 🔧 Examples

The repository includes several examples to help you get started:

### Run the Ping-Pong Server
```bash
cargo run --example unison_ping_server
```

### Run the Ping-Pong Client
```bash
# In another terminal
cargo run --example unison_ping_client
```

### Available Test Methods

The ping-pong example demonstrates these Unison Protocol methods:

- **`ping`**: Basic request-response with message echo
- **`echo`**: Echo any JSON data with optional transformations
- **`get_server_time`**: Get server timestamp and uptime

## 🧠 Testing

Run the test suite:

```bash
# Run all tests
cargo test

# Run with logging
RUST_LOG=info cargo test

# Run specific test
cargo test unison_protocol_tests
```

## 🔌 Integration

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
unison-protocol = "0.1"
tokio = { version = "1.40", features = ["full"] }
serde_json = "1.0"
```

Or install via cargo:

```bash
cargo add unison-protocol
```

### Language Support Roadmap

- ✅ **Rust**: Full support with async/await
- 🚧 **TypeScript**: Code generation (in development)
- 📅 **Python**: Planned
- 📅 **Go**: Planned  
- 📅 **JavaScript (Node.js)**: Planned

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