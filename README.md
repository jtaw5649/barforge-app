<p align="center">
  <img src="assets/og.png" alt="Barforge" width="800">
</p>

<p align="center">
  <b>Browse, install, and manage waybar modules from a central registry.</b>
</p>

<p align="center">
  <a href="#installation">Installation</a> •
  <a href="#features">Features</a> •
  <a href="#module-format">Module Format</a> •
  <a href="#development">Development</a> •
  <a href="#contributing">Contributing</a>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Rust-1.85%2B-orange?style=flat-square&logo=rust" alt="Rust">
  <img src="https://img.shields.io/badge/iced-0.14-blue?style=flat-square" alt="iced">
  <img src="https://img.shields.io/badge/License-GPL--3.0-blue?style=flat-square" alt="License">
  <img src="https://img.shields.io/github/v/release/jtaw5649/barforge-app?style=flat-square" alt="Release">
</p>

---

## Screenshots

<details open>
<summary><b>Browse Marketplace</b></summary>
<p align="center">
  <img src="assets/screenshots/marketplace-browse.png" alt="Browse Marketplace" width="800">
</p>
</details>

<details>
<summary><b>Installed Modules</b></summary>
<p align="center">
  <img src="assets/screenshots/library-installed.png" alt="Installed Modules" width="800">
</p>
</details>

<details>
<summary><b>Updates Available</b></summary>
<p align="center">
  <img src="assets/screenshots/library-update.png" alt="Updates Available" width="800">
</p>
</details>

---

## Features

🔍 **Browse Registry** — Discover modules from a central registry with search and category filtering

📦 **One-Click Install** — Install modules directly from the registry without manual configuration

🔧 **Module Management** — Enable, disable, and configure installed modules with toggle switches

⚙️ **Preferences UI** — Auto-generated settings dialogs for modules that support configuration

🔄 **Update Notifications** — Know when your installed modules have updates available

🎯 **Cross-Platform** — Built with iced for a fast, native experience on any platform

## Installation

### Pre-built Binary (Recommended)

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/jtaw5649/barforge-app/releases/latest/download/barforge-bin-installer.sh | sh
```

### Arch Linux (AUR)

```bash
yay -S barforge
```

### Flatpak (Local Build)

```bash
flatpak-builder --user --install --force-clean build flatpak/dev.barforge.Barforge.yml
flatpak run dev.barforge.Barforge
```

### Build from Source

```bash
git clone https://github.com/jtaw5649/barforge-app.git
cd barforge-app
cargo build --release
./target/release/barforge
```

### Dependencies (Build from Source)

```bash
# Arch Linux
sudo pacman -S rust dbus

# Debian/Ubuntu
sudo apt install rustc cargo libdbus-1-dev pkg-config

# Other platforms: Install Rust via rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Module Format

Modules follow a format inspired by GNOME Extensions:

```
weather-wttr@barforge/
├── metadata.json    # UUID, name, description, waybar-version
├── config.jsonc     # Default waybar config snippet
├── prefs.json       # Optional: settings schema (auto-generates UI)
├── style.css        # Optional: module CSS styling
└── scripts/         # Optional: custom module scripts
```

### metadata.json

```json
{
  "uuid": "weather-wttr@barforge",
  "name": "Weather (wttr.in)",
  "description": "Display weather using wttr.in API",
  "version-name": "1.2.0",
  "waybar-version": ["0.10", "0.11"],
  "author": { "name": "Author", "url": "https://github.com/author" },
  "category": "weather"
}
```

## Submitting a Module

1. Create your module repo with required files (see Module Format above)
2. Submit a PR to [barforge-registry-data](https://github.com/jtaw5649/barforge-registry-data)

## Development

### Testing

```bash
./scripts/test.sh
```

### Building

```bash
cargo build
cargo build --release
cargo check
cargo clippy
```

### Architecture

```
.
├── crates/
│   └── barforge-registry-types/   # Shared types (ModuleUuid, RegistryModule, etc.)
└── src/
    ├── main.rs              # Application entry point
    ├── app/                 # Elm architecture (state, message, update, view)
    ├── domain/              # InstalledModule, BarSection (app-specific types)
    ├── services/            # Registry fetch, module management
    ├── tasks.rs             # Async Task operations
    ├── theme/               # Custom theming (colors, styles)
    └── widget/              # Reusable UI components (sidebar, cards, rows)
```

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

GPL-3.0-or-later. See [LICENSE](LICENSE) for details.

## Credits

Built with [iced](https://github.com/iced-rs/iced) and [iced_aw](https://github.com/iced-rs/iced_aw).

Inspired by [GNOME Extensions](https://extensions.gnome.org), [Waybar](https://github.com/Alexays/Waybar), and the Wayland ecosystem.
