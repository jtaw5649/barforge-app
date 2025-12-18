# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.2.x   | :white_check_mark: |

## Reporting a Vulnerability

If you discover a security vulnerability in Waybar Manager, please report it responsibly:

1. **Do not** open a public issue
2. Email the maintainer directly or use GitHub's private vulnerability reporting
3. Include:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact
   - Any suggested fixes (optional)

We will acknowledge receipt within 48 hours and provide a timeline for a fix.

## Security Considerations

### Module Installation
- Modules are downloaded from GitHub repositories
- Users should review module source code before installation
- Waybar Manager merges module configs into your waybar configuration
- Scripts included in modules are made executable and run by waybar

### Network Security
- All API communication uses HTTPS
- Registry API is hosted on Cloudflare Workers
- No authentication data is stored locally

### Local Data
- Configuration files are stored in `~/.config/waybar/`
- Module data is stored in `~/.local/share/waybar-manager/`
- Cache is stored in `~/.cache/waybar-manager/`
