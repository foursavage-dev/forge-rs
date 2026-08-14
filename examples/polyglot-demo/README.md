# Polyglot Demo Project

This is a sample monorepo demonstrating Forge's multi-language build capabilities.

## Project Structure

```
polyglot-demo/
├── rust-service/    # Rust backend service
├── go-service/      # Go microservice
├── web-frontend/    # TypeScript frontend
├── docker-compose/  # Docker configuration
└── forge.toml       # Forge configuration
```

## Build with Forge

```bash
# Build all services
forge build

# View build graph
forge graph

# Run tests
forge test

# Clean build artifacts
forge clean
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
