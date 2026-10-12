# Project Vingilot — Harness Build Plan

## Document Purpose

This document is the authoritative build plan for **Project Vingilot**.

It is intended to be used by:

```text
Human developers
AI coding agents
Harness/orchestration tools
Testing automation
Packaging automation
```

The project shall be implemented incrementally, one task at a time.

---

# 1. Harness Execution Rules

The executing agent or developer shall follow these rules:

1. Work on only one task at a time.
2. Read this document before starting any task.
3. Preserve the project constraints defined in this document.
4. Do not rewrite unchanged files unless explicitly required.
5. Prefer small, focused diffs.
6. Run verification commands after each task.
7. Do not mark a task complete unless all acceptance criteria pass.
8. If blocked, document the blocker in:

```text
docs/BLOCKERS.md
```

9. If an architectural decision changes, record it in:

```text
docs/DECISIONS.md
```

10. If a task requires human approval, stop and write the question to:

```text
docs/DECISIONS_NEEDED.md
```

---

# 2. Project Summary

**Project name:** Vingilot Browser

**Primary product direction:**

```text
A native GTK4/Libadwaita web browser for Linux desktop and mobile,
using WebKitGTK as the rendering engine,
with AI assistance via MCP-style tools,
local sync bundles,
and import/export support from Firefox, Chrome, Chromium, and other browsers.
```

---

# 3. Confirmed Technical Decisions

These decisions are final for the current development cycle unless explicitly changed by the project owner.

## Engine

```text
Use WebKitGTK 6.0 as the primary browser engine.
Do not attempt full Gecko embedding in the MVP.
```

## UI

```text
GTK4
Libadwaita
Adaptive desktop/mobile layout
```

## Language

```text
Rust
```

## Storage

```text
SQLite for local persistent data
JSON/JSONL for portable sync bundles
```

## AI

```text
OpenAI-compatible API support
Local LLM support through Ollama, LM Studio, or compatible endpoints
MCP-style tool layer for agentic browser actions
```

## Sync

```text
Local sync bundles
Export/import between Vingilot instances
No cloud account requirement in the first version
```

## Browser Data Import

```text
Import from Firefox
Import from Chrome/Chromium-based browsers
Import from standard bookmark HTML/JSON files
```

## Extension Strategy

```text
Do not implement Firefox WebExtension compatibility in the first version.
Use userscripts as the first extension-like mechanism.
```

## Security Constraints

The first version shall not import or sync:

```text
Passwords
Cookies
Active sessions
Payment data
Form autofill secrets
```

API keys shall not be exported by default.

---

# 4. Target Feature Set

## Browser Core

```text
WebKitGTK page rendering
Address bar
Back/forward/reload/stop
Tabs
Profiles
Bookmarks
History
Downloads
Settings
Search engine switching
Reading Mode
True Black AMOLED mode
Kinetic scrolling
Adaptive layout
```

## Profiles

Default profiles:

```text
Standard
Work
Study
Custom
```

Each profile should isolate:

```text
Bookmarks
History
Settings
Search engines
Userscripts
Browser data directory
```

## AI

```text
Page summarization
Page question answering
Read-only page inspection
Confirmed agentic actions
MCP-style tool exposure
```

## Local Sync

```text
Export selected data to a portable bundle
Import bundle into another Vingilot instance
Optional passphrase encryption
Backup before destructive imports
```

## Import

```text
Firefox bookmarks
Firefox history
Chrome bookmarks
Chrome/Chromium history
Generic bookmark HTML
Generic bookmark JSON
```

## CLI

```text
Open/read pages
Extract text
Extract title
Extract links
Screenshot
AI summary
Export/import sync bundles
```

---

# 5. Repository Layout

The repository shall use a Cargo workspace.

```text
vingilot/
├── Cargo.toml
├── BUILD_PLAN.md
├── STATUS.md
├── rust-toolchain.toml
├── .gitignore
├── scripts/
│   ├── check.sh
│   ├── run.sh
│   └── test.sh
├── docs/
│   ├── ARCHITECTURE.md
│   ├── DECISIONS.md
│   ├── DECISIONS_NEEDED.md
│   ├── BLOCKERS.md
│   ├── SYNC_BUNDLE.md
│   └── IMPORT_EXPORT.md
├── crates/
│   ├── vingilot-core/
│   ├── vingilot-engine/
│   ├── vingilot-ui/
│   ├── vingilot-ai/
│   └── vingilot-cli/
└── assets/
    ├── css/
    ├── icons/
    └── reader/
```

---

# 6. Crate Responsibilities

## vingilot-core

```text
Data models
SQLite repositories
Profiles
Bookmarks
History
Settings
Search engines
Reading list
Sync bundle logic
Import/export logic
```

## vingilot-engine

```text
Browser engine abstraction
WebKitGTK implementation
Page metadata
JavaScript execution
Profile/data directory integration
```

## vingilot-ui

```text
GTK4/Libadwaita interface
Tabs
Address bar
Sidebar
Settings
Reading Mode
Theme system
Import/export wizards
AI panel
```

## vingilot-ai

```text
AI provider abstraction
OpenAI-compatible client
Local LLM client
Tool definitions
Tool dispatcher
MCP server
Confirmation policy
```

## vingilot-cli

```text
Terminal commands
Headless or remote operations
Sync/import/export commands
AI commands
```

---

# 7. Quality Gates

Every task must satisfy the following where applicable:

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --workspace
cargo build --workspace
```

A helper script shall exist:

```text
scripts/check.sh
```

It should run:

```bash
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --workspace
```

---

# 8. Status Tracking

The repository shall contain:

```text
STATUS.md
```

The harness should update task checkboxes after completion.

Example:

```markdown
- [x] TASK-001: Bootstrap Cargo workspace
- [ ] TASK-002: Add quality gate scripts
```

Commit messages should use the format:

```text
TASK-001: Bootstrap Cargo workspace
```

---

# 9. Global Agent Context

When using an AI harness, prepend this context to task prompts:

```text
CONTEXT:
Project Vingilot is a native Linux browser.
Stack: Rust, GTK4, Libadwaita, WebKitGTK 6.0, SQLite.
Confirmed direction: WebKitGTK MVP, local sync bundles, import from Firefox/Chrome/Chromium/HTML, userscripts instead of Firefox extensions, MCP-style AI tools, no password/cookie sync in first version.
Output rules:
- Provide only required code.
- Include file paths.
- Do not rewrite unchanged files.
- Keep explanation minimal.
- Use stable Rust.
- Follow existing repository structure.
```

---

# 10. Build Tasks

---

## Phase 0 — Repository Bootstrap

---

### TASK-001: Bootstrap Cargo Workspace

- Status: Pending
- Depends: None
- Goal: Create the base Rust workspace.

Deliverables:

```text
Root Cargo.toml
crates/vingilot-core
crates/vingilot-engine
crates/vingilot-ui
crates/vingilot-ai
crates/vingilot-cli
.gitignore
rust-toolchain.toml
```

Acceptance criteria:

```text
cargo build --workspace succeeds.
All crates exist.
Binary crates compile with placeholder main.rs files.
Library crates compile with placeholder lib.rs files.
```

Verification:

```bash
cargo build --workspace
```

Agent prompt:

```text
TASK:
Create a Rust workspace for Project Vingilot with the crates vingilot-core, vingilot-engine, vingilot-ui, vingilot-ai, and vingilot-cli.

Requirements:
- Root Cargo.toml workspace.
- vingilot-ui and vingilot-cli are binary crates.
- Other crates are libraries.
- Add minimal placeholder source files.
- Add .gitignore for Rust.
- Add rust-toolchain.toml using stable Rust.
- Do not implement features yet.
```

---

### TASK-002: Add Quality Gate Scripts

- Status: Pending
- Depends: TASK-001
- Goal: Add scripts for formatting, linting, and testing.

Deliverables:

```text
scripts/check.sh
scripts/test.sh
scripts/run.sh
```

Acceptance criteria:

```text
scripts/check.sh runs fmt, clippy, and tests.
scripts/run.sh runs the GTK4 UI binary.
Scripts are executable.
```

Verification:

```bash
./scripts/check.sh
```

Agent prompt:

```text
TASK:
Add helper scripts for Project Vingilot.

Create:
- scripts/check.sh
- scripts/test.sh
- scripts/run.sh

Requirements:
- check.sh runs cargo fmt --check, clippy with warnings as errors, and cargo test.
- test.sh runs cargo test --workspace.
- run.sh runs the vingilot-ui binary.
- Make scripts executable.
```

---

### TASK-003: Create Architecture Documentation

- Status: Pending
- Depends: TASK-001
- Goal: Record core architecture decisions.

Deliverables:

```text
docs/ARCHITECTURE.md
docs/DECISIONS.md
```

Acceptance criteria:

```text
Architecture document describes crates and data flow.
Decision log records WebKitGTK, local sync, userscripts, and no password sync.
```

Agent prompt:

```text
TASK:
Create docs/ARCHITECTURE.md and docs/DECISIONS.md.

Requirements:
- Describe the crate structure.
- Describe the WebKitGTK-first strategy.
- Describe local sync bundles.
- Describe userscripts as the first extension-like mechanism.
- Record that passwords, cookies, and sessions are not synced in the first version.
```

---

## Phase 1 — Core Data Models

---

### TASK-010: Core Error and Result Types

- Status: Pending
- Depends: TASK-001
- Goal: Define shared error handling.

Deliverables:

```text
crates/vingilot-core/src/error.rs
crates/vingilot-core/src/lib.rs
```

Acceptance criteria:

```text
CoreError enum exists.
CoreResult<T> alias exists.
Errors cover IO, database, serialization, import, export, and validation.
```

Agent prompt:

```text
TASK:
Create shared error types in vingilot-core.

Define:
- CoreError enum
- CoreResult<T> type alias

Cover:
- IO errors
- SQLite errors
- serde errors
- validation errors
- import errors
- export errors
```

---

### TASK-011: Portable Data Models

- Status: Pending
- Depends: TASK-010
- Goal: Define core serializable models.

Deliverables:

```text
crates/vingilot-core/src/models.rs
```

Models:

```text
Profile
BookmarkNode
HistoryEntry
SearchEngine
UserSettings
ReadingListItem
SyncManifest
SyncOptions
DeviceIdentity
```

Acceptance criteria:

```text
Models derive Serialize/Deserialize.
BookmarkNode supports nested folders.
Profile supports Standard, Work, Study, and Custom types.
SyncOptions can enable/disable each data category.
```

Agent prompt:

```text
TASK:
Create portable data models in vingilot-core.

Define Rust structs for:
- Profile
- BookmarkNode
- HistoryEntry
- SearchEngine
- UserSettings
- ReadingListItem
- SyncManifest
- SyncOptions
- DeviceIdentity

Requirements:
- Use serde.
- Use uuid.
- Use chrono or time.
- BookmarkNode must support folders and nested children.
- Profile must include id, name, color/icon, data directory, search engine, and timestamps.
- SyncOptions must include switches for bookmarks, history, settings, search engines, reading list, userscripts, and API keys.
```

---

### TASK-012: SQLite Persistence Layer

- Status: Pending
- Depends: TASK-011
- Goal: Add local database support.

Deliverables:

```text
crates/vingilot-core/src/db.rs
crates/vingilot-core/src/repositories/
```

Acceptance criteria:

```text
SQLite database can be opened in user data directory.
Migrations can initialize tables.
Repositories exist for profiles, bookmarks, history, search engines, and settings.
```

Verification:

```bash
cargo test -p vingilot-core
```

Agent prompt:

```text
TASK:
Create SQLite persistence for vingilot-core.

Requirements:
- Use rusqlite or sqlx if simpler.
- Create a database module.
- Add schema migrations for:
  - profiles
  - bookmarks
  - history
  - search_engines
  - settings
  - reading_list
- Add repository functions for basic create/read/update/delete.
- Add tests using an in-memory database.
```

---

## Phase 2 — Browser Engine Layer

---

### TASK-020: Browser Engine Abstraction

- Status: Pending
- Depends: TASK-010
- Goal: Define an engine trait.

Deliverables:

```text
crates/vingilot-engine/src/lib.rs
```

Acceptance criteria:

```text
BrowserEngine trait exists.
Engine-agnostic events and metadata types exist.
```

Agent prompt:

```text
TASK:
Create a BrowserEngine trait in vingilot-engine.

Required operations:
- load_url
- reload
- go_back
- go_forward
- stop_loading
- execute_javascript
- get_page_title
- get_page_url
- zoom_in
- zoom_out
- reset_zoom
- set_profile_data_dir

Also define:
- EngineError
- NavigationEvent
- PageMetadata
```

---

### TASK-021: WebKitGTK Engine Wrapper

- Status: Pending
- Depends: TASK-020
- Goal: Implement WebKitGTK backend.

Deliverables:

```text
crates/vingilot-engine/src/webkit.rs
```

Acceptance criteria:

```text
WebKitEngine can create a WebKitGTK web view.
Basic navigation functions are implemented.
JavaScript execution API is available.
Profile data directory can be set.
```

Agent prompt:

```text
TASK:
Implement a WebKitGTK backend for vingilot-engine.

Requirements:
- Use WebKitGTK 6.0 compatible bindings.
- If a high-level Rust crate is unavailable, create a thin FFI wrapper and isolate unsafe code.
- Implement WebKitEngine satisfying BrowserEngine.
- Support creating a WebView, loading URLs, back/forward/reload/stop, and JavaScript execution.
- Support setting a profile/data directory.
```

---

## Phase 3 — Basic UI Shell

---

### TASK-030: Libadwaita Application Window

- Status: Pending
- Depends: TASK-001
- Goal: Launch a basic GTK4/Libadwaita window.

Deliverables:

```text
crates/vingilot-ui/src/main.rs
crates/vingilot-ui/src/window.rs
```

Acceptance criteria:

```text
Application opens a Libadwaita window.
Window has an empty content area.
Application ID is defined.
```

Agent prompt:

```text
TASK:
Create a basic GTK4/Libadwaita application for Vingilot.

Requirements:
- Use Adw.Application.
- Create an Adw.ApplicationWindow.
- Set application ID to io.github.vingilot.Vingilot.
- Set default title to Vingilot.
- Add an empty main content container.
```

---

### TASK-031: Central Address Bar

- Status: Pending
- Depends: TASK-030
- Goal: Add address bar and navigation buttons.

Acceptance criteria:

```text
Header bar contains central address entry.
Back, forward, reload, stop, and menu buttons exist.
Enter activates navigation logic.
```

Agent prompt:

```text
TASK:
Add a central address bar to the Vingilot window.

Requirements:
- Use Adw.HeaderBar or equivalent.
- Place Gtk.Entry centrally.
- Add back, forward, reload, stop buttons on the left.
- Add menu button on the right.
- Emit a navigation request when Enter is pressed.
```

---

### TASK-032: WebView Integration

- Status: Pending
- Depends: TASK-021, TASK-031
- Goal: Display web content.

Acceptance criteria:

```text
Address bar loads URLs.
Search queries use default search engine.
WebKit web view is visible in main content area.
```

Agent prompt:

```text
TASK:
Integrate WebKitEngine into the Vingilot UI.

Requirements:
- Place the WebKitGTK web view in the main content area.
- Connect address bar activation to load URLs.
- If input is not a URL, use DuckDuckGo search.
- Update window title when page title changes.
```

---

### TASK-033: Browser Essentials

- Status: Pending
- Depends: TASK-032
- Goal: Implement normal browser behavior.

Acceptance criteria:

```text
Back/forward/reload/stop work.
Loading state is visible.
Address bar updates on navigation.
Basic error pages are shown.
```

Agent prompt:

```text
TASK:
Implement basic browser behavior.

Add:
- loading state
- page title updates
- URL updates
- back/forward button sensitivity
- reload/stop behavior
- basic error page handling
```

---

## Phase 4 — Tabs and Adaptive UI

---

### TASK-040: Tab Model

- Status: Pending
- Depends: TASK-032
- Goal: Manage multiple tabs.

Deliverables:

```text
crates/vingilot-ui/src/tabs.rs
```

Acceptance criteria:

```text
Tab model supports new/close/switch.
Each tab owns a web view or engine handle.
Tab title and URL update.
```

Agent prompt:

```text
TASK:
Create a tab model for Vingilot.

Requirements:
- Represent tabs with IDs.
- Support creating, closing, and switching tabs.
- Store title and URL per tab.
- Integrate with WebKitEngine instances.
```

---

### TASK-041: Side Tab Sidebar

- Status: Pending
- Depends: TASK-040
- Goal: Render tabs in a collapsible sidebar.

Acceptance criteria:

```text
Sidebar displays open tabs.
Tabs can be selected.
Tabs can be closed.
New tab button exists.
Sidebar can collapse/expand.
```

Agent prompt:

```text
TASK:
Implement a collapsible side tab sidebar.

Requirements:
- Use GTK4/Libadwaita.
- Show tab title and close button.
- Add new tab button.
- Support switching tabs.
- Make sidebar collapsible/expandable.
```

---

### TASK-042: Adaptive Layout and Kinetic Scrolling

- Status: Pending
- Depends: TASK-041
- Goal: Support desktop and mobile layouts.

Acceptance criteria:

```text
Wide layout shows sidebar.
Narrow layout hides or overlays sidebar.
Scrollable areas use kinetic scrolling.
```

Agent prompt:

```text
TASK:
Make the Vingilot UI adaptive.

Requirements:
- Use Libadwaita adaptive containers.
- On wide layouts, show sidebar.
- On narrow layouts, hide or overlay sidebar.
- Enable kinetic scrolling for scrollable UI areas.
```

---

## Phase 5 — Profiles, Bookmarks, History

---

### TASK-050: Profile Backend

- Status: Pending
- Depends: TASK-012
- Goal: Implement profiles in database.

Acceptance criteria:

```text
Default profiles can be created: Standard, Work, Study, Custom.
Active profile can be selected.
Profile data directory is available to engine.
```

Agent prompt:

```text
TASK:
Implement profile backend.

Requirements:
- Store profiles in SQLite.
- Create default profiles: Standard, Work, Study, Custom.
- Store active profile selection.
- Provide per-profile data directory for WebKitGTK.
```

---

### TASK-051: Profile Switcher UI

- Status: Pending
- Depends: TASK-050, TASK-033
- Goal: Allow switching profiles.

Acceptance criteria:

```text
Profile switcher appears in header bar or sidebar.
Switching profile changes browser data context.
```

Agent prompt:

```text
TASK:
Add a profile switcher UI.

Requirements:
- Show available profiles.
- Allow switching active profile.
- Reload or create engine context for selected profile.
```

---

### TASK-052: Bookmark Backend

- Status: Pending
- Depends: TASK-012
- Goal: Store bookmarks in SQLite.

Acceptance criteria:

```text
Bookmark folders and items can be created.
Nested folders are supported.
Bookmarks can be moved, renamed, and deleted.
```

Agent prompt:

```text
TASK:
Implement bookmark backend.

Requirements:
- Store bookmarks in SQLite.
- Support folders, nested folders, and bookmark items.
- Support add, remove, rename, move, and reorder operations.
```

---

### TASK-053: Bookmark Panel UI

- Status: Pending
- Depends: TASK-052, TASK-041
- Goal: Provide Opera-style bookmark management.

Acceptance criteria:

```text
Bookmark panel displays tree.
User can add current page.
User can create folders.
User can edit and delete entries.
```

Agent prompt:

```text
TASK:
Create a bookmark panel UI.

Requirements:
- Display bookmark tree.
- Support folders and items.
- Support add current page.
- Support new folder, rename, delete.
- Use GTK4/Libadwaita.
```

---

### TASK-054: History Backend and Panel

- Status: Pending
- Depends: TASK-012, TASK-033
- Goal: Store and display history.

Acceptance criteria:

```text
Visited pages are stored in history.
History can be searched.
History can be cleared.
```

Agent prompt:

```text
TASK:
Implement history storage and a history panel.

Requirements:
- Store URL, title, visit time, and visit count.
- Add history panel.
- Support search and clear history.
```

---

## Phase 6 — Search Engines and Settings

---

### TASK-060: Search Engine Manager

- Status: Pending
- Depends: TASK-011
- Goal: Manage search engines.

Acceptance criteria:

```text
Default search engine can be changed.
Search engines include Google, Bing, DuckDuckGo, Brave, YouTube, Wikipedia.
Prefix shortcuts are parsed.
```

Agent prompt:

```text
TASK:
Implement a search engine manager.

Requirements:
- Support Google, Bing, DuckDuckGo, Brave Search, YouTube, Wikipedia.
- Store default search engine.
- Support prefixes:
  /g Google
  /b Bing
  /d DuckDuckGo
  /br Brave
  /y YouTube
  /w Wikipedia
- Parse address bar input accordingly.
```

---

### TASK-061: Settings Panel

- Status: Pending
- Depends: TASK-050, TASK-060
- Goal: Provide basic settings UI.

Acceptance criteria:

```text
Settings panel includes:
- theme
- default search engine
- profile settings
- sync/import entry points
```

Agent prompt:

```text
TASK:
Create a settings panel.

Include:
- theme selection
- default search engine
- profile information
- sync and import placeholders
```

---

## Phase 7 — Reading Mode and Themes

---

### TASK-070: Reading Mode Extraction

- Status: Pending
- Depends: TASK-033
- Goal: Extract readable article content.

Acceptance criteria:

```text
JavaScript extraction returns title, byline, content HTML, and text.
Extraction can be triggered from UI.
```

Agent prompt:

```text
TASK:
Implement Reading Mode extraction.

Requirements:
- Inject a Readability-style JavaScript extractor.
- Return JSON containing title, byline, content HTML, and text content.
- Handle pages without readable content gracefully.
```

---

### TASK-071: Reader View UI

- Status: Pending
- Depends: TASK-070
- Goal: Show clean reading interface.

Acceptance criteria:

```text
Reader view displays extracted article.
Font size can be adjusted.
Reader themes include Light, Sepia, Dark, True Black.
```

Agent prompt:

```text
TASK:
Create the Reader View UI.

Requirements:
- Display article title, byline, and content.
- Support font size controls.
- Support Light, Sepia, Dark, and True Black themes.
```

---

### TASK-072: True Black Theme

- Status: Pending
- Depends: TASK-030
- Goal: Add AMOLED-friendly theme.

Acceptance criteria:

```text
True Black theme uses pure black backgrounds where appropriate.
Theme can be selected in settings.
Reader view respects theme.
```

Agent prompt:

```text
TASK:
Implement True Black mode.

Requirements:
- Add CSS theme using pure black backgrounds where appropriate.
- Apply to window, sidebar, panels, address bar, and reader view.
- Keep text readable.
- Add theme switching logic.
```

---

## Phase 8 — Downloads and Permissions

---

### TASK-080: Basic Download Manager

- Status: Pending
- Depends: TASK-033
- Goal: Support file downloads.

Acceptance criteria:

```text
Downloads can start.
Progress is visible.
User can cancel downloads.
Download folder can be opened.
```

Agent prompt:

```text
TASK:
Implement a basic download manager.

Requirements:
- Use WebKitGTK download handling.
- Show download progress.
- Support cancel.
- Support opening download folder.
```

---

### TASK-081: Permission Prompts

- Status: Pending
- Depends: TASK-033
- Goal: Handle basic web permissions.

Acceptance criteria:

```text
Permission prompts appear for supported requests.
User can allow or deny.
Decisions can be remembered per domain.
```

Agent prompt:

```text
TASK:
Implement permission prompt handling.

Requirements:
- Support common permission requests where practical.
- Show Libadwaita dialogs.
- Store per-domain decisions.
```

---

## Phase 9 — AI and MCP

---

### TASK-090: AI Provider Abstraction

- Status: Pending
- Depends: TASK-001
- Goal: Support OpenAI-compatible endpoints.

Deliverables:

```text
crates/vingilot-ai/src/provider.rs
```

Acceptance criteria:

```text
AiProvider trait exists.
OpenAI-compatible client works with configurable base URL and model.
Local LLM endpoints are supported.
```

Agent prompt:

```text
TASK:
Create an AI provider abstraction.

Requirements:
- Define AiProvider trait.
- Support OpenAI-compatible HTTP APIs.
- Support base URL, API key, and model name.
- Support local endpoints such as Ollama and LM Studio.
```

---

### TASK-091: Browser Tool Dispatcher

- Status: Pending
- Depends: TASK-021, TASK-090
- Goal: Create internal AI tools.

Tools:

```text
get_current_url
get_page_title
get_page_text
get_dom_summary
navigate
go_back
go_forward
reload
click_selector
fill_input
scroll_page
activate_reading_mode
```

Acceptance criteria:

```text
Tools are registered.
Mutating tools require confirmation.
Read-only tools can run without confirmation.
```

Agent prompt:

```text
TASK:
Create a browser tool dispatcher for AI actions.

Required tools:
- get_current_url
- get_page_title
- get_page_text
- get_dom_summary
- navigate
- go_back
- go_forward
- reload
- click_selector
- fill_input
- scroll_page
- activate_reading_mode

Requirements:
- Mutating tools require user confirmation.
- Read-only tools do not require confirmation.
```

---

### TASK-092: MCP Server

- Status: Pending
- Depends: TASK-091
- Goal: Expose tools through MCP-compatible interface.

Acceptance criteria:

```text
MCP server lists tools.
Tool metadata includes name, description, input schema, and risk level.
Server uses stdio or Unix socket.
```

Agent prompt:

```text
TASK:
Expose Vingilot browser tools through an MCP-compatible server.

Requirements:
- Use stdio or Unix socket.
- Register tools from the tool dispatcher.
- Include metadata for each tool.
- Preserve confirmation policy for mutating tools.
```

---

### TASK-093: AI Assistant Panel

- Status: Pending
- Depends: TASK-090
- Goal: Provide user-facing AI chat panel.

Acceptance criteria:

```text
User can ask AI about current page.
AI can summarize page.
AI provider settings are editable.
```

Agent prompt:

```text
TASK:
Create an AI assistant panel.

Requirements:
- Chat interface using GTK4/Libadwaita.
- Button to summarize current page.
- Provider settings for base URL, API key, and model.
- Do not export API keys by default.
```

---

## Phase 10 — Local Sync Bundles

---

### TASK-100: Sync Manifest and Options

- Status: Pending
- Depends: TASK-011
- Goal: Define sync metadata.

Acceptance criteria:

```text
SyncManifest includes schema version, device identity, timestamp, and options.
SyncOptions controls included data types.
```

Agent prompt:

```text
TASK:
Implement sync manifest and options.

Requirements:
- SyncManifest includes schema_version, application, created_at, device identity, and options.
- SyncOptions includes flags for bookmarks, history, settings, search engines, reading list, userscripts, and API keys.
- API keys must default to excluded.
```

---

### TASK-101: Bundle Writer

- Status: Pending
- Depends: TASK-100
- Goal: Write sync bundle files.

Bundle contents:

```text
manifest.json
profiles.json
bookmarks.json
history.jsonl
settings.json
search_engines.json
reading_list.json
```

Agent prompt:

```text
TASK:
Create a sync bundle writer.

Requirements:
- Write manifest.json, profiles.json, bookmarks.json, history.jsonl, settings.json, search_engines.json, and reading_list.json.
- Use a directory-based bundle before compression.
- Validate data before writing.
```

---

### TASK-102: Bundle Reader

- Status: Pending
- Depends: TASK-101
- Goal: Read and validate sync bundles.

Acceptance criteria:

```text
Bundle reader validates schema_version.
Missing optional files are handled gracefully.
Invalid bundles produce clear errors.
```

Agent prompt:

```text
TASK:
Create a sync bundle reader.

Requirements:
- Read bundle files.
- Validate schema_version.
- Return typed models.
- Handle missing optional files gracefully.
```

---

### TASK-103: ZIP Archive Support

- Status: Pending
- Depends: TASK-102
- Goal: Export/import single-file bundles.

Acceptance criteria:

```text
Export produces .vingilot file.
Import reads .vingilot file.
Invalid archives are rejected.
```

Agent prompt:

```text
TASK:
Add ZIP support for sync bundles.

Requirements:
- Export bundle to a single .vingilot ZIP file.
- Import bundle from .vingilot ZIP file.
- Preserve internal file structure.
```

---

### TASK-104: Encrypted Bundle Support

- Status: Pending
- Depends: TASK-103
- Goal: Add passphrase encryption.

Acceptance criteria:

```text
User can encrypt bundle with passphrase.
Importer detects encryption.
Wrong passphrase produces clear error.
```

Agent prompt:

```text
TASK:
Add optional encryption to sync bundles.

Requirements:
- Use passphrase-based encryption.
- Prefer age with passphrase or AES-256-GCM with Argon2id key derivation.
- Detect encrypted bundles during import.
- Do not store passwords or cookies in bundles.
```

---

### TASK-105: Export Wizard UI

- Status: Pending
- Depends: TASK-104
- Goal: Allow users to export data.

Acceptance criteria:

```text
User can select data types.
User can choose output file.
User can enable encryption.
Export progress and result are shown.
```

Agent prompt:

```text
TASK:
Create an export wizard UI.

Requirements:
- Checkboxes for bookmarks, history, settings, search engines, reading list, userscripts.
- File chooser for output.
- Optional encryption with passphrase.
- Progress and status feedback.
```

---

### TASK-106: Import Wizard UI

- Status: Pending
- Depends: TASK-104
- Goal: Allow users to import bundles.

Acceptance criteria:

```text
User can select .vingilot file.
Encrypted bundles request passphrase.
User can choose merge/replace/new-profile import mode.
Backup is created before replace/merge.
```

Agent prompt:

```text
TASK:
Create an import wizard UI.

Requirements:
- Select .vingilot file.
- Handle encrypted bundles.
- Show bundle metadata.
- Choose import mode:
  - merge
  - replace
  - new profile
- Create backup before destructive imports.
```

---

### TASK-107: Merge and Backup Logic

- Status: Pending
- Depends: TASK-106
- Goal: Safely merge imported data.

Acceptance criteria:

```text
Bookmark duplicates are avoided where practical.
History merges by URL.
Backup exists before import.
Failed import does not corrupt current data.
```

Agent prompt:

```text
TASK:
Implement merge and backup logic for imports.

Requirements:
- Support merge, replace, and new-profile modes.
- Merge bookmarks by folder path/title and URL.
- Merge history by URL.
- Create timestamped backups before destructive operations.
```

---

## Phase 11 — Browser Import

---

### TASK-110: Firefox Bookmark Import

- Status: Pending
- Depends: TASK-052
- Goal: Import Firefox bookmarks.

Acceptance criteria:

```text
Firefox profiles can be detected.
Bookmarks can be imported from Firefox HTML/JSON export.
places.sqlite import is supported where practical.
Original Firefox files are not modified.
```

Agent prompt:

```text
TASK:
Implement Firefox bookmark import.

Requirements:
- Detect Firefox profiles on Linux.
- Support importing bookmarks from Firefox HTML or JSON export.
- Support places.sqlite by copying it to a temporary location.
- Convert to Vingilot BookmarkNode structures.
```

---

### TASK-111: Firefox History Import

- Status: Pending
- Depends: TASK-110
- Goal: Import Firefox history.

Acceptance criteria:

```text
History entries are extracted from temporary places.sqlite copy.
Timestamps are converted correctly.
Original Firefox files are not modified.
```

Agent prompt:

```text
TASK:
Implement Firefox history import.

Requirements:
- Copy places.sqlite to temporary location.
- Extract URL, title, visit count, and last visit time.
- Convert to Vingilot HistoryEntry.
```

---

### TASK-112: Chrome/Chromium Bookmark Import

- Status: Pending
- Depends: TASK-052
- Goal: Import Chromium bookmarks.

Acceptance criteria:

```text
Chrome/Chromium profile directories can be detected.
Bookmarks JSON file is parsed.
Folder hierarchy is preserved.
```

Agent prompt:

```text
TASK:
Implement Chrome/Chromium bookmark import.

Requirements:
- Detect Chrome, Chromium, Brave, Edge, and Vivaldi profile directories.
- Parse Chromium Bookmarks JSON.
- Convert to Vingilot BookmarkNode structures.
```

---

### TASK-113: Chrome/Chromium History Import

- Status: Pending
- Depends: TASK-112
- Goal: Import Chromium history.

Acceptance criteria:

```text
History SQLite file is copied before reading.
URL, title, visit count, and last visit time are extracted.
Original database is not modified.
```

Agent prompt:

```text
TASK:
Implement Chrome/Chromium history import.

Requirements:
- Copy History SQLite database to temporary location.
- Extract URL, title, visit count, last visit time.
- Convert Chromium timestamps.
```

---

### TASK-114: Generic Bookmark Import

- Status: Pending
- Depends: TASK-052
- Goal: Import from standard files.

Acceptance criteria:

```text
Netscape bookmark HTML can be imported.
JSON bookmark export can be imported.
Folder hierarchy is preserved where possible.
```

Agent prompt:

```text
TASK:
Implement generic bookmark import.

Requirements:
- Parse Netscape-format bookmark HTML.
- Parse simple JSON bookmark exports.
- Convert to Vingilot BookmarkNode structures.
```

---

### TASK-115: Import Source Selection UI

- Status: Pending
- Depends: TASK-110, TASK-112, TASK-114
- Goal: Provide unified import entry point.

Acceptance criteria:

```text
User can choose:
- Vingilot bundle
- Firefox
- Chrome/Chromium
- Brave
- Edge
- Vivaldi
- HTML bookmark file
- JSON bookmark file
```

Agent prompt:

```text
TASK:
Create an import source selection UI.

Requirements:
- Show detected browsers where possible.
- Allow manual file selection.
- Let user choose bookmarks and/or history.
- Show import progress and result.
```

---

## Phase 12 — CLI

---

### TASK-120: CLI Skeleton

- Status: Pending
- Depends: TASK-001
- Goal: Create CLI binary.

Acceptance criteria:

```text
vingilot-cli runs.
clap argument parsing exists.
Help text is clear.
```

Agent prompt:

```text
TASK:
Create the vingilot-cli skeleton using clap.

Add subcommand placeholders:
- open
- read
- text
- title
- links
- screenshot
- ai
- export
- import
```

---

### TASK-121: CLI Read/Text/Title/Links Commands

- Status: Pending
- Depends: TASK-120, TASK-021
- Goal: Provide terminal page inspection.

Acceptance criteria:

```text
CLI can fetch a URL.
CLI can output readable text.
CLI can output title and links.
```

Agent prompt:

```text
TASK:
Implement CLI read/text/title/links commands.

Requirements:
- Use WebKitGTK headless if practical.
- If headless is not practical, communicate with running GUI via Unix socket.
- For read command, extract readable article text.
```

---

### TASK-122: CLI Sync Commands

- Status: Pending
- Depends: TASK-104, TASK-120
- Goal: Allow export/import from terminal.

Commands:

```text
vingilot-cli export
vingilot-cli import
vingilot-cli import-firefox
vingilot-cli import-chrome
vingilot-cli import-bookmarks-html
```

Agent prompt:

```text
TASK:
Add sync and import commands to the CLI.

Requirements:
- Support export with optional encryption.
- Support importing .vingilot bundles.
- Support Firefox, Chrome/Chromium, and HTML bookmark imports.
- Prompt for passphrase when needed.
```

---

## Phase 13 — Userscripts

---

### TASK-130: Userscript Parser

- Status: Pending
- Depends: TASK-010
- Goal: Parse userscript metadata.

Acceptance criteria:

```text
Parser supports name, description, match, exclude, and run-at.
Invalid scripts are rejected gracefully.
```

Agent prompt:

```text
TASK:
Implement a userscript metadata parser.

Supported metadata:
- name
- description
- match
- exclude
- run-at document-start
- run-at document-end
```

---

### TASK-131: Userscript Injection

- Status: Pending
- Depends: TASK-021, TASK-130
- Goal: Inject userscripts into pages.

Acceptance criteria:

```text
Userscripts load from profile directory.
Scripts inject only on matching domains.
Scripts can be enabled/disabled.
```

Agent prompt:

```text
TASK:
Implement userscript injection.

Requirements:
- Load userscripts from profile-specific directory.
- Inject using WebKitGTK user content facilities.
- Respect match/exclude patterns.
- Allow enabling/disabling scripts.
```

---

## Phase 14 — Packaging and Polish

---

### TASK-140: Flatpak Packaging

- Status: Pending
- Depends: TASK-033
- Goal: Provide GNOME-friendly packaging.

Acceptance criteria:

```text
Flatpak manifest exists.
Application installs and launches.
Desktop entry exists.
```

Agent prompt:

```text
TASK:
Create Flatpak packaging for Vingilot.

Requirements:
- Use GNOME runtime.
- Include GTK4, Libadwaita, WebKitGTK 6.0 dependencies.
- Add desktop entry metadata.
- Use application ID io.github.vingilot.Vingilot.
```

---

### TASK-141: Web Compatibility Test Suite

- Status: Pending
- Depends: TASK-033
- Goal: Track site compatibility.

Acceptance criteria:

```text
Test list includes common sites.
Compatibility results can be recorded.
Site-specific quirks can be stored.
```

Agent prompt:

```text
TASK:
Create a web compatibility test suite.

Requirements:
- Maintain a list of common websites.
- Record pass/fail/manual notes.
- Add a site quirks model for per-domain compatibility settings.
```

---

### TASK-142: User Documentation

- Status: Pending
- Depends: TASK-105, TASK-106, TASK-115
- Goal: Document user-facing features.

Deliverables:

```text
docs/USER_GUIDE.md
docs/SYNC.md
docs/IMPORT.md
```

Agent prompt:

```text
TASK:
Create user documentation.

Cover:
- profiles
- bookmarks
- reading mode
- true black mode
- AI assistant
- export/import
- Firefox/Chrome import
- CLI usage
```

---

# 11. Milestone Acceptance Criteria

---

## Milestone A: Development Environment Complete

Tasks required:

```text
TASK-001
TASK-002
TASK-003
```

Complete when:

```text
Workspace builds.
Quality scripts work.
Architecture decisions are documented.
```

---

## Milestone B: Basic Browser Works

Tasks required:

```text
TASK-010
TASK-011
TASK-012
TASK-020
TASK-021
TASK-030
TASK-031
TASK-032
TASK-033
```

Complete when:

```text
The application loads websites.
Address bar works.
Navigation buttons work.
Basic errors are handled.
```

---

## Milestone C: Tabbed Adaptive Browser

Tasks required:

```text
TASK-040
TASK-041
TASK-042
```

Complete when:

```text
Multiple tabs work.
Sidebar works.
Layout adapts to narrow and wide sizes.
```

---

## Milestone D: Data-Driven Browser

Tasks required:

```text
TASK-050
TASK-051
TASK-052
TASK-053
TASK-054
TASK-060
TASK-061
```

Complete when:

```text
Profiles work.
Bookmarks work.
History works.
Search engine switching works.
```

---

## Milestone E: Reading and Theme Experience

Tasks required:

```text
TASK-070
TASK-071
TASK-072
```

Complete when:

```text
Reading Mode works.
True Black mode works.
Reader themes are usable.
```

---

## Milestone F: AI Assistance

Tasks required:

```text
TASK-090
TASK-091
TASK-092
TASK-093
```

Complete when:

```text
AI provider can be configured.
AI can summarize page.
MCP tools are exposed.
Confirmed agentic actions are possible.
```

---

## Milestone G: Local Sync and Import

Tasks required:

```text
TASK-100
TASK-101
TASK-102
TASK-103
TASK-104
TASK-105
TASK-106
TASK-107
TASK-110
TASK-111
TASK-112
TASK-113
TASK-114
TASK-115
```

Complete when:

```text
User can export encrypted Vingilot bundle.
User can import bundle into another instance.
User can import bookmarks/history from Firefox and Chrome/Chromium.
Backups are created before destructive imports.
```

---

## Milestone H: CLI and Userscripts

Tasks required:

```text
TASK-120
TASK-121
TASK-122
TASK-130
TASK-131
```

Complete when:

```text
CLI can read pages and output text.
CLI can export/import sync bundles.
Userscripts can be loaded and injected.
```

---

## Milestone I: Packaging and Stability

Tasks required:

```text
TASK-140
TASK-141
TASK-142
```

Complete when:

```text
Flatpak packaging exists.
Compatibility testing process exists.
User documentation exists.
```

---

# 12. Harness Prompt Template

For any task, the harness should use this structure:

```text
CONTEXT:
Project Vingilot is a native Linux browser.
Stack: Rust, GTK4, Libadwaita, WebKitGTK 6.0, SQLite.
Confirmed direction: WebKitGTK MVP, local sync bundles, import from Firefox/Chrome/Chromium/HTML, userscripts instead of Firefox extensions, MCP-style AI tools, no password/cookie sync in first version.
Output rules:
- Provide only required code.
- Include file paths.
- Do not rewrite unchanged files.
- Keep explanation minimal.
- Use stable Rust.
- Follow existing repository structure.

TASK:
[Insert task description here.]

ACCEPTANCE CRITERIA:
[Insert acceptance criteria here.]

VERIFICATION:
[Insert verification commands here.]
```

---

# 13. Recommended Completion Order

The harness should execute tasks in this order:

```text
TASK-001
TASK-002
TASK-003
TASK-010
TASK-011
TASK-012
TASK-020
TASK-021
TASK-030
TASK-031
TASK-032
TASK-033
TASK-040
TASK-041
TASK-042
TASK-050
TASK-051
TASK-052
TASK-053
TASK-054
TASK-060
TASK-061
TASK-070
TASK-071
TASK-072
TASK-080
TASK-081
TASK-090
TASK-091
TASK-092
TASK-093
TASK-100
TASK-101
TASK-102
TASK-103
TASK-104
TASK-105
TASK-106
TASK-107
TASK-110
TASK-111
TASK-112
TASK-113
TASK-114
TASK-115
TASK-120
TASK-121
TASK-122
TASK-130
TASK-131
TASK-140
TASK-141
TASK-142
```

---

# 14. Initial STATUS.md Template

Create `STATUS.md` with the following content:

```markdown
# Vingilot Build Status

## Phase 0 — Repository Bootstrap

- [ ] TASK-001: Bootstrap Cargo Workspace
- [ ] TASK-002: Add Quality Gate Scripts
- [ ] TASK-003: Create Architecture Documentation

## Phase 1 — Core Data Models

- [ ] TASK-010: Core Error and Result Types
- [ ] TASK-011: Portable Data Models
- [ ] TASK-012: SQLite Persistence Layer

## Phase 2 — Browser Engine Layer

- [ ] TASK-020: Browser Engine Abstraction
- [ ] TASK-021: WebKitGTK Engine Wrapper

## Phase 3 — Basic UI Shell

- [ ] TASK-030: Libadwaita Application Window
- [ ] TASK-031: Central Address Bar
- [ ] TASK-032: WebView Integration
- [ ] TASK-033: Browser Essentials

## Phase 4 — Tabs and Adaptive UI

- [ ] TASK-040: Tab Model
- [ ] TASK-041: Side Tab Sidebar
- [ ] TASK-042: Adaptive Layout and Kinetic Scrolling

## Phase 5 — Profiles, Bookmarks, History

- [ ] TASK-050: Profile Backend
- [ ] TASK-051: Profile Switcher UI
- [ ] TASK-052: Bookmark Backend
- [ ] TASK-053: Bookmark Panel UI
- [ ] TASK-054: History Backend and Panel

## Phase 6 — Search Engines and Settings

- [ ] TASK-060: Search Engine Manager
- [ ] TASK-061: Settings Panel

## Phase 7 — Reading Mode and Themes

- [ ] TASK-070: Reading Mode Extraction
- [ ] TASK-071: Reader View UI
- [ ] TASK-072: True Black Theme

## Phase 8 — Downloads and Permissions

- [ ] TASK-080: Basic Download Manager
- [ ] TASK-081: Permission Prompts

## Phase 9 — AI and MCP

- [ ] TASK-090: AI Provider Abstraction
- [ ] TASK-091: Browser Tool Dispatcher
- [ ] TASK-092: MCP Server
- [ ] TASK-093: AI Assistant Panel

## Phase 10 — Local Sync Bundles

- [ ] TASK-100: Sync Manifest and Options
- [ ] TASK-101: Bundle Writer
- [ ] TASK-102: Bundle Reader
- [ ] TASK-103: ZIP Archive Support
- [ ] TASK-104: Encrypted Bundle Support
- [ ] TASK-105: Export Wizard UI
- [ ] TASK-106: Import Wizard UI
- [ ] TASK-107: Merge and Backup Logic

## Phase 11 — Browser Import

- [ ] TASK-110: Firefox Bookmark Import
- [ ] TASK-111: Firefox History Import
- [ ] TASK-112: Chrome/Chromium Bookmark Import
- [ ] TASK-113: Chrome/Chromium History Import
- [ ] TASK-114: Generic Bookmark Import
- [ ] TASK-115: Import Source Selection UI

## Phase 12 — CLI

- [ ] TASK-120: CLI Skeleton
- [ ] TASK-121: CLI Read/Text/Title/Links Commands
- [ ] TASK-122: CLI Sync Commands

## Phase 13 — Userscripts

- [ ] TASK-130: Userscript Parser
- [ ] TASK-131: Userscript Injection

## Phase 14 — Packaging and Polish

- [ ] TASK-140: Flatpak Packaging
- [ ] TASK-141: Web Compatibility Test Suite
- [ ] TASK-142: User Documentation
```

---

# 15. First Action for the Harness

The harness should begin with:

```text
TASK-001: Bootstrap Cargo Workspace
```

After completion, it should update:

```text
STATUS.md
```

Then proceed to:

```text
TASK-002: Add Quality Gate Scripts
```