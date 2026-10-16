<p align="center">
  <img src="data/icons/hicolor/128x128/apps/io.github.antux912012.Vingilot.png" alt="Vingilot Logo" width="128" height="128">
</p>

<h1 align="center">Vingilot</h1>

<p align="center">
  <strong>Native GTK4 & Libadwaita Web Browser for Linux</strong><br>
  <em>“Eärendil was a mariner that tarried in Arvernien; he built a ship of timber felled in Nimbrethil to journey in; her sails he wove of silver fair, of silver were her lanterns made...”</em>
</p>

<p align="center">
  <a href="https://antux912012.github.io/arda-tech/"><img src="https://img.shields.io/badge/Ecosystem-Arda%20Tech-teal.svg" alt="Arda Tech"></a>
  <a href="https://github.com/Antux912012/Vingilot"><img src="https://img.shields.io/badge/GitHub-Vingilot%20Updates-black.svg?logo=github" alt="GitHub"></a>
  <a href="#-distribution-packages"><img src="https://img.shields.io/badge/Platform-Linux%20%7C%20Wayland-blue.svg" alt="Platform"></a>
  <a href="#1-flatpak-bundle"><img src="https://img.shields.io/badge/Flatpak-Supported-blue.svg" alt="Flatpak"></a>
  <a href="#2-debian--ubuntu-package-deb"><img src="https://img.shields.io/badge/Debian%20%2F%20Ubuntu-.deb-red.svg" alt="Debian"></a>
  <a href="#3-portable-appimage"><img src="https://img.shields.io/badge/AppImage-x86__64-orange.svg" alt="AppImage"></a>
  <a href="#architecture"><img src="https://img.shields.io/badge/Engine-WebKitGTK%206.0-green.svg" alt="Engine"></a>
  <a href="#architecture"><img src="https://img.shields.io/badge/UI-Libadwaita%20%2F%20GTK4-purple.svg" alt="UI"></a>
</p>

**Vingilot** is a modern, lightweight, and privacy-focused web browser built with **Libadwaita (GTK4)** and **WebKitGTK 6.0**, optimized for native **Wayland** compositing and hardware-accelerated **Vulkan/OpenGL** rendering.

Part of the **[Arda Tech Ecosystem](https://antux912012.github.io/arda-tech/)**, Vingilot is distributed **exclusively** in three universal Linux formats: **Flatpak**, **Debian (.deb)**, and **AppImage**. For news, release updates, and documentation, visit [Arda Tech](https://antux912012.github.io/arda-tech/) and the [GitHub Project](https://github.com/Antux912012/Vingilot).

---

> [!WARNING]
> **Work in Progress (Active Development)**
> Vingilot is currently in active development. Features are being continuously implemented, refined, and tested. While daily web browsing and UI workflows are operational, expect frequent updates and improvements.

---

## ✨ Features

- **Native Libadwaita Design**: Fluid GNOME-native interface adhering to GNOME Human Interface Guidelines (HIG).
- **Zen-Style Vertical Tabs**: Collapsible left sidebar tabs featuring website favicons, title truncation, and Zen-style top-left close badges (`×`) on hover.
- **Dynamic Start Page & Accent Sync**: Native Libadwaita Start Page with customizable GNOME color palette circles that dynamically tint the title HeaderBar and sidebar tabs.
- **Full Right-Click Context Menus**: Native WebKit developer inspector and right-click menus on web pages (Back, Forward, Reload, Copy, Inspect Element, Open Link in New Tab) plus dedicated tab context menus (Reload, Duplicate, Close Tab, Mute, Pin).
- **Vingilot Shield Ad-Blocker**: Built-in native WebKit content filtering compiled directly into the rendering engine.
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

## 📦 Distribution Packages

Vingilot is distributed in three formats: **Flatpak**, **Debian/Ubuntu (.deb)**, and standalone **AppImage**.

### 1. Flatpak Bundle
Vingilot is packaged as an isolated Flatpak bundle with Wayland, Vulkan, audio, and IPC sharing permissions:
```bash
# Install the Flatpak bundle
flatpak install io.github.antux912012.Vingilot.flatpak

# Run Vingilot
flatpak run io.github.antux912012.Vingilot
```

### 2. Debian / Ubuntu Package (`.deb`)
Native `.deb` package for Debian, Ubuntu, Linux Mint, Pop!_OS, and other Debian derivatives:
```bash
# Build the .deb package locally
./scripts/build-deb.sh

# Install the generated .deb package
sudo apt install ./dist/vingilot_0.1.3_amd64.deb

# Run Vingilot
vingilot-ui
```

### 3. Portable AppImage
Zero-install, single-file executable for 64-bit Linux:
```bash
# Build the AppImage locally
./scripts/build-appimage.sh

# Make executable and run
chmod +x ./dist/Vingilot-0.1.3-x86_64.AppImage
./dist/Vingilot-0.1.3-x86_64.AppImage
```

You can also build both `.deb` and `.AppImage` packages simultaneously using:
```bash
./scripts/build-all.sh
```

### Advanced GPU Renderer Selection
Vingilot supports explicit GTK Scene Kit (GSK) renderer overrides:
```bash
# Force Vulkan rendering
GSK_RENDERER=vulkan vingilot-ui

# Force OpenGL rendering
GSK_RENDERER=gl vingilot-ui
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

## 🌐 Arda Tech Ecosystem & Updates

- **Official Website & Tech Hub**: [Arda Tech Hub](https://antux912012.github.io/arda-tech/) — Explore related projects, articles, and documentation.
- **GitHub Repository**: [Antux912012/Vingilot](https://github.com/Antux912012/Vingilot) — Source code, issue tracker, release notes, and latest downloads.
- **Exclusive Distribution Formats**: Vingilot is distributed **exclusively** via Flatpak, Debian packages (`.deb`), and standalone AppImage binaries.

---

## 📄 License

Project Vingilot is distributed under the MIT License. See `LICENSE` for details.
