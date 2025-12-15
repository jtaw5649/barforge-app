<p align="center">
  <img src="assets/icons/hicolor/scalable/apps/org.waybar.ExtensionManager.svg" width="128" height="128" alt="Waybar Extension Manager">
</p>

<h1 align="center">Waybar Extension Manager</h1>

<p align="center">
  <b>A native GTK4 extension manager for Waybar — browse, install, and manage modules from a central registry.</b>
</p>

<p align="center">
  <a href="#installation">Installation</a> •
  <a href="#features">Features</a> •
  <a href="#module-format">Module Format</a> •
  <a href="#development">Development</a>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Rust-1.75%2B-orange?style=flat-square&logo=rust" alt="Rust">
  <img src="https://img.shields.io/badge/GTK-4.12%2B-green?style=flat-square&logo=gtk" alt="GTK4">
  <img src="https://img.shields.io/badge/License-MIT-blue?style=flat-square" alt="License">
</p>

---

## Features

🔍 **Browse Registry** — Discover modules from a central registry with search and category filtering

📦 **One-Click Install** — Install modules directly from the registry without manual configuration

🔧 **Module Management** — Enable, disable, and configure installed modules with toggle switches

⚙️ **Preferences UI** — Auto-generated settings dialogs for modules that support configuration

🔄 **Update Notifications** — Know when your installed modules have updates available

🎯 **Native Experience** — Built with libadwaita for seamless desktop integration

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
sudo pacman -S gtk4 libadwaita rust
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
2. Add `versions.json` for waybar version compatibility
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
src/
├── application.rs       # Adw.Application lifecycle
├── window.rs            # NavigationSplitView main window
├── domain/              # ModuleUuid, RegistryModule, InstalledModule
├── services/            # Registry fetch, module management
└── ui/
    ├── pages/           # BrowsePage, InstalledPage
    └── widgets/         # ModuleCard
```

## Credits

Built with [gtk4-rs](https://github.com/gtk-rs/gtk4-rs) and [libadwaita](https://gitlab.gnome.org/GNOME/libadwaita).

Inspired by [GNOME Extensions](https://extensions.gnome.org), [Waybar](https://github.com/Alexays/Waybar), and the Wayland ecosystem.
