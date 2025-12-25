# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.3.x   | :white_check_mark: |
| 0.2.x   | :x:                |

## Reporting a Vulnerability

If you discover a security vulnerability in Barforge, please report it responsibly:

1. **Do not** open a public issue
2. Email the maintainer directly or use GitHub's private vulnerability reporting
3. Include:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact
   - Any suggested fixes (optional)

We will acknowledge receipt within 48 hours and provide a timeline for a fix.

## Security Features (v0.3.0+)

### Landlock Sandbox
Install scripts run in a Landlock LSM sandbox with:
- Restricted filesystem access (read-only system paths, write to /tmp only)
- Network isolation (configurable per module)
- Process isolation

### Signature Verification
Modules from the registry are cryptographically signed:
- Ed25519 signatures via Minisign
- Signature verified before package extraction
- Compile-time embedded public key

### Revocation Checking
- Modules can be revoked if security issues are discovered
- Fail-closed by default (network errors abort installation)
- Checked before download begins

### Archive Extraction Protection
- Algebraic path normalization (prevents TOCTOU attacks)
- Path traversal attempts rejected
- Symlinks and hardlinks rejected
- 50MB size limit on packages

### Dependency Validation
- Binary dependencies checked via `which`
- Python module dependencies validated safely
- Injection-safe validation (no shell execution)

## Local Data

- Configuration files: `~/.config/barforge/`
- Module data: `~/.local/share/barforge/`
- Cache: `~/.cache/barforge/`
