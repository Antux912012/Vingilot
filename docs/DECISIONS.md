# Project Vingilot — Architectural Decisions Log

This document records the confirmed architectural decisions for Project Vingilot.

## ADR-001: WebKitGTK 6.0 as Primary Rendering Engine
- **Status:** Accepted
- **Context:** An MVP web browser on Linux requires robust standards-compliant web rendering and Wayland/GTK integration.
- **Decision:** Use WebKitGTK 6.0 bindings. Full Gecko embedding is deferred beyond MVP.

## ADR-002: GTK4 and Libadwaita for UI
- **Status:** Accepted
- **Context:** Target environment is modern Linux desktop and mobile devices.
- **Decision:** Use GTK4 with Libadwaita for adaptive layouts, modern GNOME HIG design, and responsive window scaling.

## ADR-003: Rust Workspace Architecture
- **Status:** Accepted
- **Context:** Clear separation of concerns is needed across engine, UI, domain models, CLI, and AI tools.
- **Decision:** Use a multi-crate Cargo workspace (`vingilot-core`, `vingilot-engine`, `vingilot-ui`, `vingilot-ai`, `vingilot-cli`).

## ADR-004: Local Sync Bundles over Cloud Sync
- **Status:** Accepted
- **Context:** Users require privacy-first portability across multiple machines without mandating a centralized cloud account.
- **Decision:** Implement offline sync bundles (`.vingilot`) containing structured JSON/JSONL datasets with optional passphrase-based encryption.

## ADR-005: Userscripts as First Extension Mechanism
- **Status:** Accepted
- **Context:** Implementing the complete WebExtensions API in WebKitGTK is complex for an MVP.
- **Decision:** Support userscripts as the initial extension mechanism, isolated per profile with declarative metadata (`@match`, `@run-at`).

## ADR-006: Strict Exclusion of Passwords, Cookies, and Session Tokens
- **Status:** Accepted
- **Context:** Sync and cross-browser import security.
- **Decision:** The initial release shall neither import nor export passwords, cookies, active sessions, payment details, or autofill credentials. API keys are excluded by default in export bundles.

## ADR-007: MCP-Style AI Tool Integration with Confirmation Policy
- **Status:** Accepted
- **Context:** Integrating AI capabilities without compromising user control and safety.
- **Decision:** Implement Model Context Protocol (MCP) compatible tools where read-only actions execute directly and mutating actions require user confirmation.

## ADR-008: Local Encrypted Password Manager "Círdan"
- **Status:** Accepted
- **Context:** Secure local credential management without cloud exposure.
- **Decision:** Introduce a local-only password manager named "Círdan". Passwords are stored encrypted locally in SQLite, never synced online, and can only be exported/imported via user-initiated encrypted bundles protected by a master passphrase.

## ADR-009: Incognito Mode "Navigating Shadowy Seas"
- **Status:** Accepted
- **Context:** Private browsing that leaves zero traces on the local file system.
- **Decision:** Introduce an incognito mode branded as "Navigating Shadowy Seas". In this mode:
  - Ephemeral in-memory database and WebKit ephemeral data context are used.
  - History, cookies, cache, form inputs, and reading list are never committed to disk.
  - UI indicators clearly highlight when Navigating Shadowy Seas is active.


