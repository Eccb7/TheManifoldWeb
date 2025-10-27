# Contributing to The Manifold Web

Thank you for your interest in contributing to The Manifold Web!

## Development Setup

### Prerequisites

- Rust 1.76 or later
- Python 3.10 or later
- IPFS daemon (for integration tests)
- Git

### Building the Project

```bash
# Clone the repository
git clone https://github.com/Eccb7/TheManifoldWeb.git
cd TheManifoldWeb

# Build all Rust crates
cargo build --workspace

# Install Python dependencies
pip install -r python/simulation-lab/requirements.txt
```

## Testing

### Rust Tests

```bash
# Run all tests
cargo test --workspace

# Run tests for a specific crate
cargo test -p manifold-protocol
```

### Python Tests

```bash
cd python/simulation-lab
pytest
```

## Code Style

- Follow Rust standard formatting: `cargo fmt`
- Follow Python PEP 8 style guidelines
- Add documentation comments for public APIs
- Include unit tests for new features

## Pull Request Process

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

By contributing, you agree that your contributions will be licensed under the same license as the project.
