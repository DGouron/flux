# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.2.x   | :white_check_mark: |
| < 0.2.0 | :x:                |

## Reporting a Vulnerability

**Please do not report security vulnerabilities through public GitHub issues.**

Instead, use [GitHub Security Advisories](https://github.com/DGouron/flux/security/advisories/new) to report vulnerabilities privately.

### What to Include

- Description of the vulnerability
- Steps to reproduce the issue
- Potential impact
- Suggested fix (if you have one)

### Response Timeline

- **Initial response**: Within 48 hours
- **Status update**: Within 7 days
- **Resolution target**: Within 30 days for critical issues

## What Constitutes a Security Issue

### In Scope

- Code execution vulnerabilities
- Privilege escalation
- Secrets/credentials exposure
- Path traversal vulnerabilities
- Unsafe file operations (especially `/etc/hosts` modifications)
- IPC protocol vulnerabilities

### Out of Scope

- Denial of service through resource exhaustion (CLI tool)
- Issues requiring physical access to the machine
- Social engineering attacks
- Issues in dependencies (report upstream, but let us know)

## Security Best Practices in Flux

Flux follows these security principles:

1. **No hardcoded secrets**: All credentials use environment variables or secure storage
2. **Minimal permissions**: Uses systemd capabilities instead of sudo when possible
3. **Safe file operations**: Validates paths and creates backups before modifications
4. **Input validation**: Validates all user input and IPC messages
