# Contributing to Unison Protocol

Thank you for your interest in contributing to Unison Protocol! We welcome contributions from the community and are grateful for any help you can provide.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [How to Contribute](#how-to-contribute)
- [Development Process](#development-process)
- [Pull Request Process](#pull-request-process)
- [Style Guidelines](#style-guidelines)
- [Testing](#testing)
- [Documentation](#documentation)
- [Community](#community)

## Code of Conduct

This project and everyone participating in it is governed by our [Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code. Please report unacceptable behavior to contact@chronista.club.

## Getting Started

### Prerequisites

- Rust 1.70 or higher
- Tokio 1.40 or higher
- OpenSSL or BoringSSL (for QUIC support)

### Setting up your development environment

1. Fork the repository on GitHub
2. Clone your fork locally:
   ```bash
   git clone https://github.com/your-username/unison-protocol.git
   cd unison-protocol
   ```

3. Add the upstream repository as a remote:
   ```bash
   git remote add upstream https://github.com/chronista-club/unison-protocol.git
   ```

4. Build the project:
   ```bash
   cargo build
   ```

5. Run tests to ensure everything is working:
   ```bash
   cargo test
   ```

## How to Contribute

### Reporting Bugs

Before creating bug reports, please check existing issues to avoid duplicates. When you create a bug report, include as many details as possible:

- Use a clear and descriptive title
- Describe the exact steps to reproduce the problem
- Provide specific examples to demonstrate the steps
- Describe the behavior you observed and explain why it's a problem
- Explain the behavior you expected to see
- Include your environment details (OS, Rust version, etc.)

### Suggesting Enhancements

Enhancement suggestions are tracked as GitHub issues. When creating an enhancement suggestion:

- Use a clear and descriptive title
- Provide a detailed description of the proposed enhancement
- Include examples of how the feature would be used
- Explain why this enhancement would be useful to most users

### Your First Code Contribution

Unsure where to begin? Look for issues labeled:

- `good first issue` - Good for newcomers
- `help wanted` - Extra attention is needed
- `documentation` - Documentation improvements

## Development Process

### Branching Strategy

- `main` - The main development branch
- Feature branches should be created from `main`
- Use descriptive branch names: `feature/add-new-handler`, `fix/connection-timeout`, etc.

### Commit Messages

We follow the [Conventional Commits](https://www.conventionalcommits.org/) specification:

```
<type>(<scope>): <subject>

<body>

<footer>
```

Types:
- `feat`: A new feature
- `fix`: A bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `perf`: Performance improvements
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

Examples:
```
feat(network): add retry logic for QUIC connections
fix(parser): handle edge case in KDL parsing
docs: update API documentation for UnisonStream
```

## Pull Request Process

1. Ensure your code adheres to the project's style guidelines
2. Update documentation as needed
3. Add tests for new functionality
4. Ensure all tests pass: `cargo test`
5. Run formatting: `cargo fmt`
6. Run linting: `cargo clippy`
7. Update the CHANGELOG.md with your changes (if applicable)
8. Create a pull request with a clear title and description

### PR Review Process

- At least one maintainer review is required
- All CI checks must pass
- Code coverage should not decrease
- Documentation must be updated for new features

## Style Guidelines

### Rust Code Style

- Follow standard Rust conventions and idioms
- Use `cargo fmt` to format your code
- Use `cargo clippy` to catch common mistakes
- Prefer explicit error handling over `unwrap()`
- Write descriptive variable and function names
- Add comments for complex logic

### Documentation Style

- Use clear, concise language
- Include code examples where appropriate
- Keep README and other docs up to date
- Document all public APIs

## Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run tests with output
RUST_LOG=debug cargo test -- --nocapture

# Run integration tests
cargo test --test quic_integration_test
```

### Writing Tests

- Write unit tests for all new functionality
- Include integration tests for complex features
- Aim for at least 80% code coverage
- Test edge cases and error conditions

### Benchmarking

```bash
# Run benchmarks
cargo bench

# Run specific benchmark
cargo bench bench_name
```

## Documentation

- Document all public APIs using Rust doc comments
- Include examples in documentation
- Keep the README up to date
- Update architectural documentation for significant changes

### Building Documentation

```bash
# Build and open documentation
cargo doc --open

# Build documentation with private items
cargo doc --document-private-items
```

## Community

### Communication Channels

- GitHub Issues: Bug reports and feature requests
- GitHub Discussions: General discussions and Q&A
- Discord: [Join our Discord server](https://discord.gg/unison-protocol) (Coming soon)

### Getting Help

If you need help, you can:

1. Check the [documentation](https://docs.rs/unison-protocol)
2. Search existing issues
3. Ask in GitHub Discussions
4. Reach out on Discord

## Recognition

Contributors will be recognized in:
- The project's CHANGELOG.md
- Special mentions in release notes
- Our contributors list

## License

By contributing to Unison Protocol, you agree that your contributions will be licensed under the MIT License.

---

Thank you for contributing to Unison Protocol! Your efforts help make this project better for everyone. 🎵