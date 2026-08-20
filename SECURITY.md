# Security Policy

> 🌐 **Translations & Contributions:** Want to translate or improve this document in your language? See our [Translation Guidelines](TRANSLATION.md).

## Supported Versions

Fish currently provides security updates for the latest supported release series.

| Version | Supported |
| ------- | --------- |
| 0.2.x   | :white_check_mark: |

Older release series may no longer receive security updates. Users are encouraged to upgrade to the latest supported version whenever possible.

## Reporting a Vulnerability

If you discover a potential security vulnerability in Fish, please **do not open a public issue** or publicly disclose the vulnerability before a fix or coordinated disclosure has been arranged.

Instead, report the vulnerability privately by email:

**foursavage@proton.me**

Please include as much of the following information as possible:

- A description of the vulnerability
- Steps to reproduce the issue
- Affected Fish versions or components
- Potential security impact
- Relevant logs, proof-of-concept code, or other technical details
- Any proposed mitigation or fix

Please do not include unnecessary personal or sensitive information in your report.

### Response Process

Fish Maintainers will:

1. Acknowledge receipt of the report as soon as reasonably possible, with a target of 48 hours.
2. Review and validate the reported vulnerability.
3. Determine the severity and affected components.
4. Work with the reporter to understand the issue and possible mitigations.
5. Develop and test an appropriate fix.
6. Coordinate disclosure and release of the fix when appropriate.

Response times may vary depending on the severity, complexity, and availability of the maintainers.

### Coordinated Disclosure

Security vulnerabilities should be handled privately until a fix or appropriate mitigation is available.

Fish Maintainers may coordinate disclosure with the reporter, including:

- The affected versions
- The fixed version
- The nature and impact of the vulnerability
- Available mitigations
- Appropriate credit for the reporter, if requested

The security of Fish users and contributors will be prioritized when determining the disclosure timeline.

## Security Features

Fish includes security-related capabilities designed to improve the integrity and security of builds and execution environments.

### Artifact Signing & Verification

- Ed25519 cryptographic signing of build artifacts
- SBOM generation using supported formats such as SPDX and CycloneDX
- Automated artifact integrity verification
- Source-to-build traceability

### Dependency Vulnerability Scanning

- Vulnerability scanning for supported dependency ecosystems
- Severity-based security policies
- CVSS-based vulnerability assessment
- Automated security reports

### Secret Management

Fish supports integration with external secret-management systems where configured, including:

- HashiCorp Vault
- AWS Secrets Manager
- Kubernetes Secrets

### VFS Security

Fish provides security controls for filesystem access and isolated execution environments, including:

- Capability-based filesystem permissions
- Isolated worker environments
- Sandboxed execution for supported plugins

Security capabilities may vary between Fish components and configurations. Users should consult the relevant documentation before relying on a specific security feature.

## Secure Development Practices

Fish aims to follow secure development practices, including:

- **Memory safety:** Prefer safe Rust abstractions and prohibit `unsafe` code in crates where `#![forbid(unsafe_code)]` is applicable.
- **Security review:** Security-sensitive changes should receive appropriate review.
- **Dependency auditing:** Dependencies should be regularly reviewed and checked for known vulnerabilities.
- **Input validation:** External and untrusted input should be validated before processing.
- **Least privilege:** Components should operate with only the permissions they require.
- **Secure defaults:** Security-sensitive functionality should use safe defaults where practical.
- **Security testing:** Security-sensitive functionality should include appropriate tests and regression coverage.

## Security Best Practices for Users

### Production Deployment

Users deploying Fish in production should:

1. Keep Fish updated to the latest supported release.
2. Verify artifact signatures where artifact signing is available.
3. Run dependency and security checks as part of the build pipeline.
4. Use a dedicated secret-management solution for sensitive credentials.
5. Follow the principle of least privilege.
6. Protect signing keys and rotate them according to organizational security requirements.
7. Review security logs and audit information where available.

### CI/CD Integration

For CI/CD environments:

1. Verify artifact integrity before deployment.
2. Run security checks as part of CI pipelines.
3. Use short-lived credentials where possible.
4. Grant CI jobs only the permissions they require.
5. Avoid exposing secrets to untrusted build steps.
6. Keep CI runners and dependencies up to date.

### Distributed Workers

When using distributed workers:

1. Use encrypted communication such as TLS.
2. Authenticate worker connections.
3. Restrict worker permissions.
4. Isolate worker environments where possible.
5. Limit filesystem access to the directories and resources required by the worker.

## Security Advisories

Security advisories may be published through the GitHub Security Advisories system:

https://github.com/requla11/fish/security/advisories

Security advisories may include information about:

- Affected versions
- Severity
- Security impact
- Mitigations
- Fixed versions

## Acknowledgments

We thank all security researchers and contributors who responsibly report vulnerabilities and help improve the security of Fish.

Security researchers may receive credit for valid vulnerability reports unless they prefer to remain anonymous.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for the full license text.
