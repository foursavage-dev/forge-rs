# Security Policy

## Supported Versions

Currently, only the latest version of Forge is supported with security updates.

| Version | Supported          |
| ------- | ------------------- |
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

If you discover a potential security vulnerability in Forge, please **do not** open a public issue.

Instead, send an email to **foursavage@proton.me** with the following information:

- A description of the vulnerability
- Steps to reproduce the vulnerability
- Affected versions
- Potential impact
- Any proposed mitigation or fix

The Foursavage security team will:

1. Acknowledge receipt of your report within 48 hours
2. Provide a detailed response within 7 days
3. Work with you to understand and validate the issue
4. Determine the severity and timeline for disclosure
5. Coordinate the release of a fix with you

## Security Features

Forge includes several security-related features:

### Artifact Signing & Verification
- Ed25519 cryptographic signing of build artifacts
- SBOM generation (SPDX/CycloneDX formats)
- Automated verification of artifact integrity
- Source-to-build chain tracking

### Dependency Vulnerability Scanning
- Multi-backend vulnerability scanning (Rust, NPM, Maven)
- Severity-based blocking policies
- CVSS score tracking
- Automated security reports

### Secret Management
- Secure secret injection with HashiCorp Vault
- AWS Secrets Manager integration
- Kubernetes secrets support
- Audit trail for secret usage

### VFS Security
- Capability-based filesystem permissions
- Isolated worker environments
- Sandbox execution for custom plugins

## Secure Development Practices

Forge follows secure development practices:

- **No unsafe code**: `#![forbid(unsafe_code)]` in relevant crates
- **Security review**: All code undergoes security review
- **Dependency auditing**: Regular dependency updates and vulnerability scanning
- **Input validation**: Comprehensive input validation across all backends
- **Least privilege**: Minimal permissions for all operations

## Security Best Practices for Users

### Production Deployment
1. Use artifact signing for all production builds
2. Enable dependency vulnerability scanning
3. Use secret management instead of environment variables
4. Keep Forge updated to the latest version
5. Review and rotate signing keys regularly

### CI/CD Integration
1. Verify artifact signatures before deployment
2. Run security scans in CI pipelines
3. Use temporary credentials with minimal permissions
4. Enable audit logging for all operations

### Distributed Workers
1. Use TLS for all worker communications
2. Authenticate all worker connections
3. Enable VFS for secure file access
4. Isolate worker environments

## Security Advisories

Past security advisories will be published in the [Security Advisories](https://github.com/foursavage-dev/forge-rs/security/advisories) section.

## Acknowledgments

We thank all security researchers who help keep Forge secure. Please review our [Security Policy](https://github.com/foursavage-dev/forge-rs/blob/main/SECURITY.md) for details.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
