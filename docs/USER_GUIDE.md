# Vingilot User Guide

Welcome to **Project Vingilot**, the native GTK4/Libadwaita web browser designed for modern Linux desktop and mobile environments.

---

## 1. Getting Started

### Launching the Browser
- **Normal Launch:**
  ```bash
  cargo run -p vingilot-ui
  ```
  Or using the helper script:
  ```bash
  ./scripts/run.sh
  ```
- **Incognito Mode:**
  Launch Vingilot with zero local disk footprint (in-memory SQLite, ephemeral WebKit storage):
  ```bash
  ./scripts/run.sh --incognito
  ```

---

## 2. Browser Features

### Adaptive UI & Side Tabs
- **Collapsible Sidebar:** Toggle the side tab bar to display open tabs vertically with quick close buttons.
- **Adaptive Layout:** Dynamically collapses on mobile/narrow screens (< 720px) and expands on desktop displays.
- **Kinetic Scrolling:** Smooth touch and gesture scrolling across scrollable views.

### Profiles
Switch effortlessly between isolated browsing contexts:
- **Standard:** Everyday browsing.
- **Work:** Professional accounts and bookmarks.
- **Study:** Reference material and reading lists.
- **Custom:** Tailored environments.

### Bookmarks & History
- **Opera-style Hierarchical Tree:** Create folders, nest bookmarks, and quickly save the current page.
- **History Search:** Fast live search through previous page titles and URLs with visit counts.

### Reading Mode & True Black AMOLED Theme
- **Reading Mode:** Strips ads and navigational noise to extract clean article text, author byline, and reading time estimates.
- **Reader Themes:** Switch between `Light`, `Sepia`, `Dark`, and `True Black`.
- **True Black Mode:** Pure `#000000` AMOLED styling across window backgrounds, sidebars, and input entries.

### Círdan — Local Encrypted Password Manager
- **Privacy-first:** Passwords are kept securely on your device, encrypted with AES-256-GCM.
- **Zero Cloud Exposure:** Passwords are never sent over the network or uploaded to cloud accounts.
- **Export/Import:** Easily move your credentials to another Vingilot instance via encrypted bundle archives with a master passphrase.

### Search Engine Shortcuts
Type shortcuts directly into the unified address bar:
- `/d <query>` — DuckDuckGo (Default)
- `/g <query>` — Google
- `/b <query>` — Bing
- `/br <query>` — Brave Search
- `/y <query>` — YouTube
- `/w <query>` — Wikipedia

---

## 3. AI Assistant & Model Context Protocol (MCP)

- **AI Panel:** Chat with local LLMs (Ollama, LM Studio) or OpenAI-compatible endpoints.
- **Page Summarization:** One-click article and page summary generation.
- **Agentic MCP Tools:** Built-in tool dispatcher supporting safe read-only analysis and user-confirmed mutating actions.

---

## 4. Userscripts

Add custom `.user.js` files to your profile directory to extend site functionality:
- Supports `@match`, `@exclude`, and `@run-at` (`document-start`, `document-end`).
