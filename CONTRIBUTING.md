# Contributing to Waybar Manager

Thank you for your interest in contributing to Waybar Manager!

## Development Setup

### Prerequisites
- Rust (stable toolchain)
- System dependencies:
  - `libdbus-1-dev` (Debian/Ubuntu) or `dbus` (Arch)
  - `pkg-config`

### Building
```bash
cargo build
cargo run
```

### Testing
```bash
cargo test
cargo clippy
cargo fmt --check
```

## Submitting Changes

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/my-feature`)
3. Make your changes
4. Ensure tests pass (`cargo test`)
5. Ensure clippy passes (`cargo clippy -- -D warnings`)
6. Ensure formatting is correct (`cargo fmt`)
7. Commit your changes with a descriptive message
8. Push to your fork
9. Open a Pull Request

## Code Style

- Follow Rust conventions and idioms
- Use `cargo fmt` for formatting
- Address all `cargo clippy` warnings
- Write tests for new functionality
- Keep commits focused and atomic

## Reporting Issues

When reporting bugs, please include:
- Your operating system and version
- Rust version (`rustc --version`)
- Steps to reproduce the issue
- Expected vs actual behavior
- Any relevant error messages or logs

## Adding Modules to the Registry

To submit a new waybar module to the registry, please open an issue or PR at:
https://github.com/jtaw5649/waybar-modules-registry
