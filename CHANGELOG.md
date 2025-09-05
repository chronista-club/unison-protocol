# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2025-01-05

### Added
- 🎵 Initial release of Unison Protocol
- KDL-based schema definition system for type-safe communication
- WebSocket client and server implementation with async/await support
- Schema parser with comprehensive type validation
- Code generation framework for Rust (TypeScript support in development)
- Core protocol types: `UnisonMessage`, `UnisonResponse`, `UnisonError`
- Network abstractions for client/server communication
- Complete documentation and protocol specification
- Example implementations:
  - `unison_ping_server.rs` - Full-featured ping-pong server
  - `unison_ping_client.rs` - Comprehensive test client
- Schema definitions:
  - `unison_core.kdl` - Core protocol schema
  - `ping_pong.kdl` - Example ping-pong protocol
  - `common.kdl` - Common type definitions
- GitHub Actions CI/CD pipeline with multi-Rust version testing
- MIT License for open source distribution

### Features
- **Type Safety**: Compile-time and runtime protocol validation
- **Multi-Language Support**: Rust implementation with TypeScript planned
- **Real-time Communication**: WebSocket-based bidirectional messaging
- **Schema Validation**: KDL-based protocol definitions with validation
- **Code Generation**: Automatic client/server code generation
- **Async Support**: Built with tokio for high-performance async I/O
- **Developer Experience**: Comprehensive error handling and debugging support

### Technical Details
- **Dependencies**: 
  - `kdl` 4.6+ for schema parsing
  - `tokio` 1.40+ for async runtime  
  - `tokio-tungstenite` 0.24+ for WebSocket support
  - `serde` 1.0+ for serialization
  - Full dependency list in `Cargo.toml`
- **Build System**: Custom build script for code generation
- **Testing**: Comprehensive unit tests and integration examples
- **Documentation**: Full API documentation and usage examples

### Repository Structure
```
unison-protocol/
├── .github/workflows/ci.yml    # GitHub Actions CI
├── .gitignore                  # Git ignore rules
├── Cargo.toml                  # Rust package configuration
├── LICENSE                     # MIT License
├── README.md                   # Project documentation
├── CHANGELOG.md                # This file
├── PROTOCOL_SPEC.md            # Protocol specification
├── build.rs                    # Build script for code generation
├── src/                        # Source code
│   ├── lib.rs                  # Library entry point
│   ├── core/                   # Core protocol types
│   ├── parser/                 # Schema parsing logic
│   ├── codegen/                # Code generation framework
│   └── network/                # Network implementation
├── schemas/                    # Protocol schema definitions
│   ├── unison_core.kdl         # Core protocol schema
│   ├── ping_pong.kdl           # Example protocol
│   └── common.kdl              # Common types
└── examples/                   # Usage examples
    ├── unison_ping_server.rs   # Example server
    └── unison_ping_client.rs   # Example client
```

### Next Steps
- Publish to crates.io as `unison-protocol`
- TypeScript code generation implementation
- Additional language bindings (Python, Go, JavaScript)
- Enhanced schema validation features
- Performance optimizations and benchmarks

### Migration Notes
This is the initial independent release extracted from the diarkis-tools project. No migration is required for new users.