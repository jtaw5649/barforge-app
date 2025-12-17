<p align="center">
  <img src="assets/icons/hicolor/scalable/apps/org.waybar.ExtensionManager.svg" width="128" height="128" alt="Waybar Extension Manager">
</p>
<h1 align="center">Waybar Manager</h1>

<p align="center">
  <b>An iced module manager for Waybar — browse, install, and manage modules from a central registry.</b>
</p>

<p align="center">
  <a href="#installation">Installation</a> •
  <a href="#features">Features</a> •
  <a href="#module-format">Module Format</a> •
  <a href="#development">Development</a>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Rust-1.75%2B-orange?style=flat-square&logo=rust" alt="Rust">
  <img src="https://img.shields.io/badge/iced-0.14-blue?style=flat-square" alt="iced">
  <img src="https://img.shields.io/badge/License-GPL--3.0-blue?style=flat-square" alt="License">
</p>

---

## Features

🔍 **Browse Registry** — Discover modules from a central registry with search and category filtering

📦 **One-Click Install** — Install modules directly from the registry without manual configuration

🔧 **Module Management** — Enable, disable, and configure installed modules with toggle switches

⚙️ **Preferences UI** — Auto-generated settings dialogs for modules that support configuration

🔄 **Update Notifications** — Know when your installed modules have updates available

🎯 **Cross-Platform** — Built with iced for a fast, native experience on any platform

## Installation

### Build from Source

```bash
git clone https://github.com/jtaw5649/waybar-manager.git
cd waybar-manager
cargo build --release
./target/release/waybar-manager-bin
```

### Dependencies

```bash
# Arch Linux
sudo pacman -S rust

# Other platforms: Install Rust via rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Module Format

Modules follow a format inspired by GNOME Extensions:

```
weather-wttr@waybar-modules/
├── metadata.json    # UUID, name, description, waybar-version
├── config.jsonc     # Default waybar config snippet
├── prefs.json       # Optional: settings schema (auto-generates UI)
├── style.css        # Optional: module CSS styling
└── scripts/         # Optional: custom module scripts
```

### metadata.json

```json
{
  "uuid": "weather-wttr@waybar-modules",
  "name": "Weather (wttr.in)",
  "description": "Display weather using wttr.in API",
  "version-name": "1.2.0",
  "waybar-version": ["0.10", "0.11"],
  "author": { "name": "Author", "url": "https://github.com/author" },
  "category": "weather"
}
```

## Registry

The module registry is hosted on GitHub Pages:

| URL | Purpose |
|-----|---------|
| `https://waybar-modules.github.io/registry/index.json` | Module listings |
| `https://waybar-modules.github.io/registry/schemas/` | JSON Schema validation |

### Submitting a Module

1. Create your module repo with required files
3. Submit a PR to the registry repo

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
│   └── waybar-registry-types/   # Shared types (ModuleUuid, RegistryModule, etc.)
└── src/
    ├── main.rs              # Application entry point
    ├── app/                 # Elm architecture (state, message, update, view)
    ├── domain/              # InstalledModule, BarSection (app-specific types)
    ├── services/            # Registry fetch, module management
    ├── tasks.rs             # Async Task operations
    ├── theme/               # Custom theming (colors, styles)
    └── widget/              # Reusable UI components (sidebar, cards, rows)
```

## Credits

Built with [iced](https://github.com/iced-rs/iced) and [iced_aw](https://github.com/iced-rs/iced_aw).

Inspired by [GNOME Extensions](https://extensions.gnome.org), [Waybar](https://github.com/Alexays/Waybar), and the Wayland ecosystem.
