# Security Policy

## Supported Versions

We release patches for security vulnerabilities. Which versions are eligible for receiving such patches depends on the CVSS v3.0 Rating:

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |
| < 0.1   | :x:                |

## Reporting a Vulnerability

We take the security of Unison Protocol seriously. If you have discovered a security vulnerability in our project, we appreciate your help in disclosing it to us in a responsible manner.

### How to Report

**Please do not report security vulnerabilities through public GitHub issues.**

Instead, please report them via email to:
- **Email**: security@chronista.club
- **Subject Line**: [SECURITY] Unison Protocol - [Brief Description]

### What to Include

Please include the following information in your report:

1. **Type of issue** (e.g., buffer overflow, SQL injection, cross-site scripting, etc.)
2. **Full paths of source file(s)** related to the manifestation of the issue
3. **Location** of the affected source code (tag/branch/commit or direct URL)
4. **Step-by-step instructions** to reproduce the issue
5. **Proof-of-concept or exploit code** (if possible)
6. **Impact** of the issue, including how an attacker might exploit it
7. **Any special configuration** required to reproduce the issue

### Response Timeline

- **Initial Response**: Within 48 hours, we will acknowledge receipt of your vulnerability report
- **Assessment**: Within 7 days, we will provide an initial assessment of the report
- **Resolution**: We will work on fixes and keep you informed of our progress
- **Disclosure**: We will coordinate with you on public disclosure timing

## Security Update Process

When we receive a security bug report, we will:

1. **Confirm** the problem and determine affected versions
2. **Audit** code to find any similar problems
3. **Develop** fixes for all supported versions
4. **Release** patches as soon as possible

## Security-Related Configuration

### QUIC/TLS Configuration

Unison Protocol uses QUIC with TLS 1.3 for secure communication. Ensure you:

1. Use strong, properly generated certificates in production
2. Never commit private keys to version control
3. Rotate certificates regularly
4. Use the provided certificate generation scripts only for development

### Best Practices

When using Unison Protocol:

1. **Keep Dependencies Updated**: Regularly update to the latest version
2. **Use Secure Defaults**: Don't disable security features unless absolutely necessary
3. **Validate Input**: Always validate and sanitize input data
4. **Limit Exposure**: Use firewalls and network policies to limit exposure
5. **Monitor**: Implement logging and monitoring for suspicious activities

## Known Security Considerations

### Development Certificates

The project includes utilities for generating self-signed certificates for development:
- These are located in `assets/certs/`
- **Never use these certificates in production**
- Always generate new certificates for production deployments

### Dependencies

We regularly audit our dependencies for known vulnerabilities using:
- `cargo audit`
- GitHub's Dependabot alerts

## Security Features

Unison Protocol includes several security features:

1. **TLS 1.3 Encryption**: All QUIC connections are encrypted
2. **Certificate Validation**: Proper certificate chain validation
3. **Input Validation**: Protocol-level message validation
4. **Memory Safety**: Leveraging Rust's memory safety guarantees
5. **Type Safety**: Strong typing throughout the protocol implementation

## Disclosure Policy

When we discover or are made aware of security vulnerabilities:

1. We will promptly investigate and validate the issue
2. Develop and test appropriate fixes
3. Release patches for all supported versions
4. Publish a security advisory with:
   - Description of the vulnerability
   - CVSS score
   - Affected versions
   - Patched versions
   - Workarounds (if any)
   - Credits to the reporter

## Contact

For any security-related questions or concerns:
- **Email**: security@chronista.club
- **GitHub Security Advisories**: [View Advisories](https://github.com/chronista-club/unison-protocol/security/advisories)

## Acknowledgments

We would like to thank the following for their responsible disclosure and contribution to the security of Unison Protocol:

*This list will be updated as we receive and address security reports.*

---

Thank you for helping keep Unison Protocol and its users safe!