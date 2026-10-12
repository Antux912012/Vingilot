# Vingilot 🚢

> *“Eärendil was a mariner that tarried in Arvernien; he built a ship of timber felled in Nimbrethil to journey in; her sails he wove of silver fair, of silver were her lanterns made...”*

[![Status: Work in Progress](https://img.shields.io/badge/Status-Work_in_Progress-amber.svg)](#status)
[![Platform](https://img.shields.io/badge/Platform-Linux%20%7C%20Wayland%20%7C%20Flatpak-blue.svg)](#installation)
[![Engine](https://img.shields.io/badge/Engine-WebKitGTK%206.0-green.svg)](#architecture)
[![UI](https://img.shields.io/badge/UI-Libadwaita%20%2F%20GTK4-purple.svg)](#architecture)

**Vingilot** is a modern, lightweight, and privacy-focused web browser built with **Libadwaita (GTK4)** and **WebKitGTK 6.0**, optimized for native **Wayland** compositing and hardware-accelerated **Vulkan** rendering.

---

> [!WARNING]
> **Work in Progress (Active Development)**
> Vingilot is currently in active development. Features are being continuously implemented, refined, and tested. While daily web browsing and UI workflows are operational, expect frequent updates and improvements.

---

## ✨ Features

- **Native Libadwaita Design**: Fluid GNOME-native interface adhering to GNOME Human Interface Guidelines (HIG).
- **Wayland & Vulkan Acceleration**: Native Wayland protocol integration and high-performance GPU scene-graph rendering powered by GTK Scene Kit (GSK) and Mesa Vulkan drivers (`libvulkan` / `anv`).
- **Header Bar 3-Dots Menu (`⋮`)**: Convenient top-right menu button aligned with window controls providing quick access to:
  - **Preferences**: Deep customization including OLED True Black themes, search engines, hardware acceleration toggles, and privacy controls.
  - **About Dialog**: System info, versioning, and project credits.
  - **Interactive Page Zoom**: In-menu `−` / `+` scaling controls with real-time percentage indicators and one-click reset.
  - **New Tab & Incognito Windows**: Fast tab navigation and isolated ephemeral browsing.
  - **History & Bookmarks**: Quick navigation access.
- **Shadowy Seas (Incognito Mode)**: Ephemeral private browsing with strict cookie isolation and zero disk footprint.
- **Local AI Assistant Integration**: Configured to connect with local Ollama or OpenAI-compatible endpoints.
- **Adaptive Layout**: Responsive side-tab sidebar collapsing gracefully across different screen sizes.

---

## 📦 Flatpak Distribution

Vingilot is packaged as an isolated, secure Flatpak bundle with Wayland, DRI/Vulkan, audio, and IPC sharing permissions.

### Option 1: Install Prebuilt Flatpak Bundle

Download `io.github.antux912012.Vingilot.flatpak` from the [GitHub Releases](https://github.com/Antux912012/Vingilot/releases) page and install it:

```bash
# Install the Flatpak bundle
flatpak install io.github.antux912012.Vingilot.flatpak

# Run Vingilot
flatpak run io.github.antux912012.Vingilot
```

### Option 2: Run with Specific GPU Renderers

Vingilot supports explicit renderer selection via GTK Scene Kit:

```bash
# Force Vulkan rendering
flatpak run --env=GSK_RENDERER=vulkan io.github.antux912012.Vingilot

# Force modern OpenGL (ngl) rendering
flatpak run --env=GSK_RENDERER=ngl io.github.antux912012.Vingilot
```

---

## 🛠️ Building from Source

### Prerequisites

Ensure you have the GNOME 50 Platform SDK and Flatpak builder installed:

```bash
flatpak install flathub org.gnome.Platform//50 org.gnome.Sdk//50 org.freedesktop.Sdk.Extension.rust-stable//25.08
sudo apt install flatpak-builder
```

### Build & Install with Flatpak Builder

```bash
# Clone the repository
git clone https://github.com/Antux912012/Vingilot.git
cd Vingilot

# Build and install locally for the current user
flatpak-builder --user --install --force-clean build-dir flatpak/io.github.antux912012.Vingilot.yml
```

### Native Cargo Development

```bash
# Quality checks (fmt, clippy, unit tests)
./scripts/check.sh

# Run development build
cargo run -p vingilot-ui -- "https://duckduckgo.com"
```

---

## 🏛️ Project Architecture

```
Vingilot/
├── crates/
│   ├── vingilot-core/     # SQLite persistence, data models, bookmarks, history, crypto
│   ├── vingilot-engine/   # WebKitGTK 6.0 engine abstraction and downloads
│   ├── vingilot-ai/       # AI provider client and MCP dispatcher
│   ├── vingilot-cli/      # Command-line interface
│   └── vingilot-ui/       # Libadwaita window shell, 3-dots popover, preferences, zoom
├── flatpak/
│   └── io.github.antux912012.Vingilot.yml  # Flatpak packaging manifest
├── data/                  # Desktop file, metainfo XML, Papirus icons
└── docs/                  # Architecture and development documentation
```

---

## 📄 License

Project Vingilot is distributed under the MIT License. See `LICENSE` for details.
