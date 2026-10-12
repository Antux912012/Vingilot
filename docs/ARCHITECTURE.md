# Project Vingilot — Architecture

## 1. Overview

**Project Vingilot** is a native GTK4/Libadwaita web browser for Linux desktop and mobile.
It is built with Rust and utilizes WebKitGTK 6.0 as its rendering engine.
It features AI assistance via an MCP-style tool layer, portable local sync bundles, and browser data import capabilities.

## 2. Crate Responsibilities

The codebase is structured as a Cargo workspace with five distinct crates:

### `vingilot-core` (Library)
The central data and domain logic layer:
- Data models (Profiles, Bookmarks, History, Search Engines, Settings, Reading List, Sync Manifest).
- SQLite persistence and repositories.
- Sync bundle reader/writer and encryption logic.
- Browser import pipelines (Firefox, Chromium/Chrome, Netscape HTML, JSON).

### `vingilot-engine` (Library)
The browser engine abstraction:
- `BrowserEngine` trait defining navigation, JavaScript execution, page metadata extraction, and zooming.
- WebKitGTK 6.0 engine implementation with per-profile data isolation and event dispatch.

### `vingilot-ui` (Binary)
The native user interface built on GTK4 and Libadwaita:
- Adaptive desktop and mobile layouts.
- Header bar with unified address bar and navigation controls.
- Tab bar and collapsible side tab panel.
- Reader View and True Black AMOLED theme.
- Management panels for Bookmarks, History, Profiles, Settings, and AI.

### `vingilot-ai` (Library)
AI integration and agentic capabilities:
- `AiProvider` trait supporting OpenAI-compatible endpoints and local LLMs (Ollama, LM Studio).
- Tool dispatcher with strict read-only vs. mutating action distinction.
- MCP (Model Context Protocol) server over stdio or Unix sockets.
- Confirmation workflows for mutating browser actions.

### `vingilot-cli` (Binary)
Command-line interface providing headless and utility operations:
- Page inspection, text, title, and link extraction.
- Sync bundle export and import operations.
- Direct browser import commands.

## 3. WebKitGTK-First Strategy

- WebKitGTK 6.0 serves as the primary rendering engine for Vingilot.
- Full Gecko embedding is deferred; WebKitGTK provides tight integration with the GNOME / Libadwaita ecosystem and hardware acceleration on Linux.

## 4. Local Sync Bundles

- Vingilot uses offline, local portable bundles (`.vingilot`) rather than requiring a cloud account or third-party sync server.
- Bundles can be exported and imported between devices with optional passphrase-based encryption (Argon2id + AES-256-GCM / age).
- Automatic timestamped backups are created prior to destructive imports or replacements.

## 5. Extension Mechanism: Userscripts

- Full WebExtensions support is out of scope for the initial release.
- Userscripts (`@match`, `@exclude`, `@run-at`) serve as the primary extension mechanism, loaded per profile and injected via WebKitGTK user content APIs.

## 6. Privacy & Security Constraints

- The sync bundle format and import tools deliberately exclude sensitive credential data:
  - No passwords
  - No session cookies
  - No active session tokens
  - No payment or card information
  - No form autofill credentials
- AI API keys are excluded by default during sync bundle exports.
