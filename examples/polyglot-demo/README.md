# Polyglot Demo Project

> 🌐 **Translations & Contributions:** Want to translate or improve this document in your language? See our [Translation Guidelines](TRANSLATION.md).

This is a sample monorepo demonstrating fish's multi-language build capabilities.

## Project Structure

```text
polyglot-demo/
├── rust-service/    # Rust backend service
├── go-service/      # Go microservice
├── web-frontend/    # TypeScript frontend
├── docker-compose/  # Docker configuration
└── fish.toml        # fish configuration
```

## Build with fish

```bash
# Build all services
fish build

# View build graph
fish graph

# Run tests
fish test

# Clean build artifacts
fish clean
```

## Language Support

- **Rust**: Backend service with Cargo
- **Go**: Microservice with Go modules
- **TypeScript**: Frontend with npm/yarn
- **Docker**: Container orchestration

## Features Demonstrated

- Multi-language dependency management
- Parallel build execution
- Cross-language caching
- Build graph visualization
