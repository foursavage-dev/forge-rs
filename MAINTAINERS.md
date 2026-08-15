# Maintainers

This document lists the maintainers of the Forge project and their responsibilities.

## Core Maintainers

### Foursavage Dev Team
- **Role**: Project Lead
- **Responsibilities**:
  - Overall project direction
  - Release management
  - Security oversight
  - Final approval on PRs
- **Contact**: team@foursavage.dev

## Active Maintainers

### Backend Maintainers

#### Rust Backend
- **Responsibilities**:
  - Rust backend development
  - Cargo integration
  - Performance optimization
- **Review areas**: `forge-backend-rust/`, `forge-core/`

#### C/C++ Backend
- **Responsibilities**:
  - C/C++ backend development
  - gcc/clang/msvc integration
  - Header dependency tracking
- **Review areas**: `forge-backend-cc/`

#### Go Backend
- **Responsibilities**:
  - Go backend development
  - go.mod integration
  - Module support
- **Review areas**: `forge-backend-go/`

#### TypeScript/JS Backend
- **Responsibilities**:
  - TypeScript/JS backend development
  - package.json integration
  - Monorepo support
- **Review areas**: `forge-backend-ts/`

#### Python Backend
- **Responsibilities**:
  - Python backend development
  - pyproject.toml integration
  - Virtual environment support
- **Review areas**: `forge-backend-py/`

#### Java Backend
- **Responsibilities**:
  - Java backend development
  - Maven/Gradle integration
  - Kotlin support
- **Review areas**: `forge-backend-java/`

#### .NET Backend
- **Responsibilities**:
  - .NET backend development
  - csproj/sln integration
  - Multi-target support
- **Review areas**: `forge-backend-dotnet/`

#### Swift Backend
- **Responsibilities**:
  - Swift backend development
  - Package.swift integration
  - iOS/macOS support
- **Review areas**: `forge-backend-swift/`

#### Dart Backend
- **Responsibilities**:
  - Dart backend development
  - pubspec.yaml integration
  - Flutter support
- **Review areas**: `forge-backend-dart/`

#### Zig Backend
- **Responsibilities**:
  - Zig backend development
  - build.zig integration
  - Cross-compilation
- **Review areas**: `forge-backend-zig/`

#### Docker Backend
- **Responsibilities**:
  - Docker backend development
  - Dockerfile integration
  - Container support
- **Review areas**: `forge-backend-docker/`

### Feature Maintainers

#### Cache & CAS
- **Responsibilities**:
  - Cache implementation
  - CAS engine
  - Remote cache
- **Review areas**: `forge-cache/`, `forge-cas/`, `forge-remote-cache/`

#### Distributed Execution
- **Responsibilities**:
  - Worker daemon
  - Cluster execution
  - VFS implementation
- **Review areas**: `forge-worker/`

#### Security
- **Responsibilities**:
  - Artifact signing
  - Vulnerability scanning
  - Secret management
- **Review areas**: `forge-signing/`, `forge-security/`, `forge-secrets/`

#### CI/CD
- **Responsibilities**:
  - CI generator
  - Multi-platform support
  - Template system
- **Review areas**: `forge-ci-generator/`, `forge-multiplatform/`, `forge-templates/`

#### CLI & UX
- **Responsibilities**:
  - CLI implementation
  - User experience
  - Documentation
- **Review areas**: `forge-cli/`, `docs/`

### Infrastructure Maintainers

- **Responsibilities**:
  - CI/CD pipelines
  - Repository management
  - Release automation
  - Infrastructure security
- **Review areas**: `.github/`, infrastructure config

## Contributor Roles

### Triagers
- Review and triage issues
- Label issues appropriately
- Guide contributors
- Ensure issue quality

### Contributors
- Submit pull requests
- Review PRs in their area of expertise
- Participate in discussions
- Help with documentation

## Becoming a Maintainer

### Requirements
- Active contributor for at least 3 months
- Demonstrated expertise in a specific area
- Consistent code quality
- Good communication skills
- Alignment with project goals

### Process
1. Express interest to current maintainers
2. Current maintainers review contributions
3. Community approval (if applicable)
4. Invitation to join maintainer team
5. Onboarding process

## Maintainer Responsibilities

### Code Review
- Review PRs in area of expertise
- Provide constructive feedback
- Ensure code quality standards
- Test changes if applicable

### Issue Management
- Triage new issues
- Provide guidance to contributors
- Identify critical issues
- Coordinate with other maintainers

### Release Management
- Participate in release planning
- Test release candidates
- Update documentation
- Communicate changes

### Community Management
- Help contributors
- Answer questions
- Foster inclusive environment
- Resolve conflicts

## Decision Making

### Consensus-Based
- Major decisions require consensus among core maintainers
- Technical decisions in area of expertise delegated to relevant maintainers
- Emergency decisions made by project lead

### Conflict Resolution
- Discuss in private channels first
- Escalate to core maintainers if needed
- Final decision by project lead if consensus cannot be reached

## Removing Maintainers

Maintainers may be removed for:
- Inactivity for 6+ months without notice
- Repeated violation of code of conduct
- Deliberate harm to the project
- Loss of interest

Process:
1. Discussion with maintainer
2. Warning if applicable
3. Removal if necessary
4. Announcement to team (if applicable)

## Recognition

Maintainers are recognized in:
- README.md
- GitHub team membership
- Release notes
- Project documentation

## Contact

- **General inquiries**: team@foursavage.dev
- **Security issues**: security@foursavage.dev
- **Code of conduct**: conduct@foursavage.dev

## Acknowledgments

We thank all maintainers and contributors who make Forge possible.
