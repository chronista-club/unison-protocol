# 🎵 Unison Protocol Specification

**Version**: 1.0.0  
**Date**: 2025-01-04  
**Status**: Draft

## Overview

Unison Protocol is a KDL-based type-safe communication framework designed for real-time bidirectional communication between clients and servers. The protocol enables automatic code generation for multiple programming languages while maintaining strong type safety and comprehensive error handling.

## Goals and Design Principles

### Primary Goals

1. **Type Safety**: Ensure compile-time and runtime type checking across all supported languages
2. **Developer Experience**: Provide a simple, intuitive API with comprehensive error messages
3. **Multi-language Support**: Generate idiomatic code for multiple programming languages
4. **Real-time Communication**: Support bidirectional communication with low latency
5. **Extensibility**: Allow easy addition of new methods, types, and protocols

### Design Principles

- **Schema-first**: Protocol definitions drive implementation, not the other way around
- **Async-first**: Built with async/await patterns as the foundation
- **Error-resilient**: Comprehensive error handling and recovery mechanisms
- **Transport-agnostic**: Support multiple transport layers (WebSocket, TCP, etc.)
- **Version-compatible**: Forward and backward compatibility support

## Protocol Structure

### Hierarchy

```
Protocol
├── Metadata (name, version, namespace, description)
├── Types (custom type definitions)
├── Messages (structured data definitions)
└── Services
    └── Methods (RPC endpoints with request/response schemas)
```

### Protocol Definition Format

Unison Protocol uses KDL (KDL Document Language) for schema definitions:

```kdl
protocol "service-name" version="1.0.0" {
    namespace "com.example.service"
    description "Service description"
    
    // Type definitions
    // Message definitions
    // Service definitions
}
```

## Core Protocol Messages

### UnisonMessage

The standard message format for all Unison Protocol communications:

```rust
struct UnisonMessage {
    id: String,           // Unique message identifier
    method: String,       // RPC method name
    payload: JsonValue,   // Method parameters as JSON
    timestamp: DateTime,  // Message creation timestamp
    version: String,      // Protocol version (default: "1.0.0")
}
```

### UnisonResponse

The standard response format:

```rust
struct UnisonResponse {
    id: String,                    // Corresponding request message ID
    success: bool,                 // Operation success indicator
    payload: Option<JsonValue>,    // Response data as JSON
    error: Option<String>,         // Error message if operation failed
    timestamp: DateTime,           // Response creation timestamp
    version: String,               // Protocol version
}
```

### UnisonError

Structured error information:

```rust
struct UnisonError {
    code: String,                  // Error code identifier
    message: String,               // Human-readable error message
    details: Option<JsonValue>,    // Additional error context
    timestamp: DateTime,           // Error occurrence timestamp
}
```

## Schema Definition Language

### Basic Types

Unison Protocol supports the following basic types:

| Type | Description | Rust Mapping | TypeScript Mapping |
|------|-------------|--------------|---------------------|
| `string` | UTF-8 text | `String` | `string` |
| `number` | Numeric values | `f64` | `number` |
| `bool` | Boolean | `bool` | `boolean` |
| `timestamp` | ISO-8601 datetime | `DateTime<Utc>` | `string` |
| `json` | Arbitrary JSON | `serde_json::Value` | `any` |
| `array` | List of items | `Vec<T>` | `T[]` |

### Field Modifiers

- `required=true`: Field must be present (default: false)
- `default=value`: Default value for optional fields
- `description="text"`: Field documentation

### Example Schema

```kdl
protocol "user-management" version="1.0.0" {
    namespace "com.example.users"
    description "User management service"
    
    message "User" {
        description "User account information"
        field "id" type="string" required=true description="Unique user identifier"
        field "username" type="string" required=true description="User login name"
        field "email" type="string" required=true description="User email address"
        field "created_at" type="timestamp" required=true description="Account creation time"
        field "is_active" type="bool" required=false default=true description="Account active status"
    }
    
    service "UserService" {
        description "User account management operations"
        
        method "create_user" {
            description "Create a new user account"
            request {
                field "username" type="string" required=true
                field "email" type="string" required=true
                field "password" type="string" required=true
            }
            response {
                field "user" type="User" required=true
                field "session_token" type="string" required=true
            }
        }
        
        method "get_user" {
            description "Retrieve user information by ID"
            request {
                field "user_id" type="string" required=true
            }
            response {
                field "user" type="User" required=true
            }
        }
        
        method "list_users" {
            description "List users with optional filtering"
            request {
                field "filter" type="string" required=false
                field "limit" type="number" required=false default=50
                field "offset" type="number" required=false default=0
            }
            response {
                field "users" type="array" item_type="User" required=true
                field "total_count" type="number" required=true
            }
        }
    }
}
```

## Network Protocol

### Transport Layer

Unison Protocol is transport-agnostic but primarily designed for WebSocket communication:

- **WebSocket**: Real-time bidirectional communication (primary)
- **TCP**: Direct socket communication (planned)
- **HTTP**: Request-response pattern (planned)

### Message Flow

1. **Connection Establishment**
   - Client initiates connection to server
   - Optional handshake exchange for version negotiation

2. **Method Invocation**
   - Client sends `UnisonMessage` with method name and parameters
   - Server processes request and sends `UnisonResponse`
   - Errors are returned as `UnisonResponse` with `success: false`

3. **Connection Management**
   - Heartbeat/ping mechanism for connection health
   - Graceful disconnection handling

### Error Handling

#### Client-side Errors
- Connection failures
- Timeout errors
- Serialization/deserialization errors
- Protocol version mismatches

#### Server-side Errors
- Method not found
- Invalid parameters
- Processing failures
- Resource limitations

#### Error Response Format

```json
{
  "id": "request-message-id",
  "success": false,
  "error": "Method not found: unknown_method",
  "timestamp": "2025-01-04T10:30:00Z",
  "version": "1.0.0"
}
```

## Code Generation

### Rust Code Generation

Generated Rust code includes:

- **Type Definitions**: Structs with Serde annotations
- **Client Traits**: Async methods for each service method
- **Server Traits**: Handler registration for methods
- **Validation**: Request/response validation logic

Example generated code:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
    #[serde(default = "default_is_active")]
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserResponse {
    pub user: User,
    pub session_token: String,
}

#[async_trait]
pub trait UserServiceClient {
    async fn create_user(&self, request: CreateUserRequest) -> Result<CreateUserResponse, NetworkError>;
    async fn get_user(&self, user_id: String) -> Result<User, NetworkError>;
    async fn list_users(&self, filter: Option<String>, limit: Option<i32>, offset: Option<i32>) -> Result<ListUsersResponse, NetworkError>;
}
```

### TypeScript Code Generation (Planned)

Generated TypeScript code will include:

- **Interface Definitions**: TypeScript interfaces for all types
- **Client Classes**: Promise-based client implementations
- **Type Guards**: Runtime type validation
- **Error Types**: Structured error handling

## Security Considerations

### Authentication and Authorization

- Protocol-level authentication not specified (transport-layer responsibility)
- Service-level authorization through custom handlers
- Session management through application-specific tokens

### Input Validation

- Automatic validation of required fields
- Type checking for all parameters
- Custom validation through handler implementation

### Transport Security

- TLS/WSS recommended for production use
- Certificate validation and pinning
- Connection encryption and integrity

## Performance Characteristics

### Message Size

- JSON-based serialization
- Typical message overhead: 100-200 bytes
- Payload size limited by transport layer

### Latency

- WebSocket: Sub-millisecond protocol overhead
- Network latency determines overall performance
- Async processing eliminates blocking operations

### Throughput

- Limited by transport layer and handler implementation
- Concurrent request handling through async runtime
- Connection pooling for high-load scenarios

## Versioning and Compatibility

### Protocol Versioning

- Semantic versioning (MAJOR.MINOR.PATCH)
- Version specified in protocol definition
- Version negotiation during handshake

### Backward Compatibility

- New optional fields: Compatible
- New required fields: Breaking change
- New methods: Compatible
- Method signature changes: Breaking change

### Forward Compatibility

- Unknown fields ignored during deserialization
- Unknown methods return "method not found" error
- Version mismatch handling

## Implementation Guidelines

### Client Implementation

1. **Connection Management**: Automatic reconnection, connection pooling
2. **Request Correlation**: Match requests with responses using message IDs
3. **Error Handling**: Proper error propagation and user feedback
4. **Timeout Handling**: Request timeouts and retry logic

### Server Implementation

1. **Handler Registration**: Type-safe handler registration
2. **Concurrent Processing**: Async request processing
3. **Resource Management**: Connection limits and cleanup
4. **Logging and Monitoring**: Request/response logging and metrics

## Future Enhancements

### Planned Features

- **Streaming Support**: Server-sent events and bidirectional streaming
- **Schema Evolution**: Runtime schema updates and migration
- **Compression**: Message compression for large payloads
- **Batch Operations**: Multiple operations in single request

### Language Support Expansion

- Python client/server generation
- Go client/server generation
- Java client generation
- C# client generation

## Appendix

### References

- [KDL Specification](https://kdl.dev/)
- [WebSocket Protocol (RFC 6455)](https://tools.ietf.org/html/rfc6455)
- [JSON Schema](https://json-schema.org/)

### Change Log

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2025-01-04 | Initial specification |

---

*This specification is a living document and will be updated as the protocol evolves.*