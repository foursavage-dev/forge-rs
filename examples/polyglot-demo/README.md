# Polyglot Demo Project

> Ã°Å¸Å’Â **Translations & Contributions:** Want to translate or improve this document in your language? See our [Translation Guidelines](../../TRANSLATION.md).

This is a sample monorepo demonstrating fish's multi-language build capabilities.

## Project Structure

```
polyglot-demo/
Ã¢â€Å“Ã¢â€â‚¬Ã¢â€â‚¬ rust-service/    # Rust backend service
Ã¢â€Å“Ã¢â€â‚¬Ã¢â€â‚¬ go-service/      # Go microservice
Ã¢â€Å“Ã¢â€â‚¬Ã¢â€â‚¬ web-frontend/    # TypeScript frontend
Ã¢â€Å“Ã¢â€â‚¬Ã¢â€â‚¬ docker-compose/  # Docker configuration
Ã¢â€â€Ã¢â€â‚¬Ã¢â€â‚¬ fish.toml       # fish configuration
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
