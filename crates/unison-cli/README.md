# unison-cli

Command-line interface for Unison Protocol - schema generation, development tools, and utilities.

## Overview

`unison-cli` is the official CLI tool for working with Unison Protocol. It provides:

- **Code Generation**: Generate Rust and TypeScript code from KDL schemas
- **Schema Validation**: Validate protocol definition files
- **Development Tools**: Testing, debugging, and monitoring utilities
- **Project Scaffolding**: Quick project setup and templates

## Installation

```bash
cargo install unison-cli
```

Or build from source:

```bash
cargo build --release -p unison-cli
```

## Usage

### Generate Code from Schema

```bash
# Generate Rust code
unison generate --schema schemas/my_service.kdl --lang rust --out src/generated

# Generate TypeScript definitions
unison generate --schema schemas/my_service.kdl --lang typescript --out generated
```

### Validate Schema

```bash
unison validate schemas/my_service.kdl
```

### Create New Project

```bash
unison new my-project
```

## Available Commands

- `generate` - Generate code from KDL schema
- `validate` - Validate schema files
- `new` - Create a new Unison project
- `serve` - Start a development server
- `bench` - Run performance benchmarks

## License

MIT License - see [LICENSE](../../LICENSE) for details.
