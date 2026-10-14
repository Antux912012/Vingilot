# Vingilot Browser — Master Engineering Plan

## Project context

You are the original developer of Vingilot, a desktop web browser hosted at:

https://github.com/Antux912012/Vingilot

You have already created the existing codebase. Your task is to evolve that project into a reliable, privacy-conscious desktop browser with the everyday functionality users expect from Firefox, Zen Browser and GNOME Web.

This is an implementation task, not a request for a conceptual proposal or a complete rewrite.

The repository is the source of truth. Preserve working functionality, existing architecture and established design decisions wherever possible.

## Non-negotiable development rules

1. Inspect the complete repository before modifying code.
2. Identify the programming language, GUI toolkit, WebKitGTK version, application entry point, build system, dependency versions, persistence layer and existing feature implementations.
3. Inspect the README, dependency manifests, build configuration, tests and relevant source files.
4. Produce a concise audit identifying existing features, incomplete features, defects and architectural limitations.
5. Do not assume a feature is missing until you have searched for its implementation.
6. Do not replace the application with a new implementation, switch frameworks or introduce a second application architecture without demonstrating why it is necessary.
7. Do not invent APIs. Verify that every WebKitGTK function and extension capability exists in the version targeted by this project.
8. Prefer small, coherent changes that preserve existing functionality.
9. After each implementation phase, build the application and run the relevant tests. Fix regressions before proceeding.
10. Do not claim a feature is implemented merely because its button or menu item exists. Verify its actual behavior.
11. Keep dependencies minimal and document any new dependencies.
12. Never leave core features as placeholder dialogs, empty callbacks, simulated functionality or TODO comments.
13. Do not silently remove existing features to make the build pass.
14. Do not stop after writing a plan. Once the audit is complete, begin implementing the approved first phase and continue through the roadmap, reporting any genuine blockers.

# Phase 0 — Repository audit and architecture

First, examine the current repository and establish:

- Application structure and startup sequence.
- Main window and browser view implementation.
- How tabs and web views are created and destroyed.
- How navigation events and browser actions are handled.
- How bookmarks, folders, history and settings are persisted.
- Whether right-click context menus already exist and how they are constructed.
- Whether downloads are functional.
- How normal and private browsing contexts are created.
- Whether ad blocking, content filters or WebExtensions already exist.
- Which WebKitGTK API version is required and which version is available in the target environment.
- Existing build, lint, formatting and test commands.
- Known bugs, duplicated logic, missing callbacks and incomplete interfaces.

Deliver an audit containing:
1. Current architecture.
2. Features that already work.
3. Features that are partially implemented or missing.
4. Risks and compatibility constraints.
5. Files likely to require modification.
6. A phased implementation plan based on the actual source code.

Do not fabricate filenames or line numbers. Identify them by inspecting the repository.

# Phase 1 — Complete essential browser functionality

## 1.1 Webpage context menus

Implement a proper right-click menu using the facilities provided by the existing GUI toolkit and WebKitGTK.

The menu must adapt to the item under the pointer.

For ordinary page content:
- Back, forward and reload.
- Save page or print, if supported by the existing application.
- Select all.
- View source, if implemented.
- Inspect element only if a legitimate developer-tools implementation is available.

For links:
- Open link in a new tab.
- Open link in a new window where supported.
- Open link in a private window.
- Copy link address.
- Download linked content where appropriate.

For images:
- Open image in a new tab.
- Save image.
- Copy image address.
- Copy image if supported.

For selected text:
- Copy.
- Search the selected text using the configured search engine.

Do not display irrelevant options when the pointer is not over the corresponding object. Disable actions that cannot be performed.

Ensure that the menu does not trigger twice or interfere with WebKit's native input, selection or accessibility behavior.

## 1.2 Tab context menu and management

Implement:
- New tab.
- Duplicate tab.
- Reload tab.
- Mute or unmute audio if supported.
- Pin or unpin tab, if supported by the existing tab architecture.
- Move tab or reorder tabs.
- Close tab.
- Close other tabs.
- Close tabs to the right.
- Reopen the most recently closed tab.
- Restore tabs after a normal application restart, subject to privacy rules.

Keep tab selection, active web view, navigation state and visual indicators synchronized.

Handle the final-tab case correctly.

## 1.3 Bookmark management

Bookmarks and bookmark folders are a high-priority feature.

Implement complete CRUD operations: create, read, update and delete.

Required functionality:
- Bookmark the current page.
- Edit a bookmark's title and URL.
- Create bookmark folders.
- Rename existing folders.
- Delete folders.
- Move bookmarks between folders.
- Move folders where the data model permits nesting.
- Reorder bookmarks and folders.
- Add bookmarks directly to a selected folder.
- Create folders from the bookmark manager.
- Search bookmarks.
- Handle empty folders.
- Provide clear confirmation when deleting a non-empty folder.

Define the deletion policy explicitly. If a folder contains bookmarks or subfolders, either require confirmation before recursively deleting it or provide a safe option to move its contents elsewhere.

Renaming must update the existing folder record, not delete the folder and create a replacement that loses its contents or identifiers.

Use stable IDs for bookmarks and folders. Preserve parent-child relationships and ordering across application restarts.

Prevent duplicate IDs, orphaned bookmarks, invalid parent references and accidental deletion of unrelated data.

If bookmarks use a database, implement appropriate transactions and migration logic. If they use a file or another persistence layer, retain the existing approach unless there is a compelling reason to change it.

The UI must refresh immediately after changes, and changes must survive a restart.

## 1.4 Bookmark context menus

Right-clicking a bookmark should provide:
- Open.
- Open in a new tab.
- Open in a private tab or window.
- Edit bookmark.
- Move to folder.
- Delete bookmark.

Right-clicking a folder should provide:
- Open all bookmarks where appropriate.
- Add bookmark.
- New subfolder if nesting is supported.
- Rename folder.
- Move folder if supported.
- Delete folder.

Support keyboard navigation, focus handling and clear confirmation dialogs.

## 1.5 Navigation, history and downloads

Audit existing behavior and complete missing functionality:
- Back, forward, reload and stop loading.
- URL entry and search fallback.
- Correct loading indicators and page titles.
- Secure-connection indicators based on actual TLS information.
- Visited-page history with timestamps and appropriate deduplication.
- History search and deletion.
- Clear history for a chosen time range.
- Download progress, cancellation, failure handling and opening completed downloads.
- Permission requests for notifications, location, camera and microphone where supported.
- Graceful handling of failed navigation, unsupported URLs and certificate errors.

Never bypass TLS errors silently.

Persist normal browsing history appropriately, but never mix private navigation into normal history.

# Phase 2 — Implement genuine private browsing

Incognito mode must be a real isolated browsing session, not a cosmetic mode.

## 2.1 WebKit architecture

Use the WebKitGTK 6.0 API appropriate to the actual installed version.

Investigate WebKitWebsiteDataManager and WebKitWebContext, including the supported ephemeral construction APIs.

Create an isolated private browsing context using WebKit's ephemeral website-data facilities wherever supported.

Private tabs and windows must not reuse the normal profile's persistent website-data manager.

Verify isolation of:
- Cookies.
- HTTP and disk cache.
- Local storage.
- IndexedDB.
- Other persistent website storage supported by the engine.
- Service workers and related site data where applicable.

Do not assume that setting a private-mode Boolean, hiding the history panel or deleting cookies after navigation creates a private session.

If the current architecture shares a WebKitWebContext between tabs, design a safe way to separate normal and private sessions without breaking existing tabs.

## 2.2 User-visible behavior

Provide:
- A New Private Window or equivalent entry point.
- A clear private-browsing visual indicator.
- Private tabs that use the private context.
- Correct handling of opening links in private browsing.
- An appropriate private start page.
- Consistent theme and keyboard shortcuts.

Keep bookmarks and user preferences persistent only when the user explicitly chooses to save or change them. Never save private URLs into ordinary history.

Downloads initiated privately require an explicit, documented policy. Files intentionally saved by the user may remain on disk, but private browsing records and metadata must not be persisted unnecessarily.

## 2.3 Session destruction and verification

When the last private window closes:
- Release private web views and their associated session resources.
- Ensure no private browsing session is restored at the next launch.
- Verify that private cookies and storage are not visible to normal tabs.
- Avoid writing private URLs to logs, crash reports or persistent session files.

Document any data that cannot be guaranteed to disappear, including files explicitly downloaded by the user and information retained by websites or network providers.

Do not promise anonymity or protection from network monitoring. Private browsing primarily prevents local persistence and cross-session leakage.

Write integration tests that prove:
1. A normal session's cookie is not present in a new private session.
2. A cookie set in private browsing is not visible in the normal session.
3. Private browsing history is absent from normal history.
4. Private storage is not restored after closing the private session.
5. Normal browsing remains functional after private windows close.

# Phase 3 — Reliable ad and tracker blocking

This is a core requirement. Build a real blocker compatible with the actual WebKitGTK engine used by Vingilot.

Do not assume that Chromium extensions, Chrome's declarativeNetRequest API, Firefox's WebRequest API or Firefox's extension packaging work unchanged in WebKitGTK.

## 3.1 Native blocking architecture

First investigate WebKitGTK's native content-filtering support, especially:
- WebKitUserContentFilterStore.
- WebKitUserContentFilter.
- WebKitUserContentManager.
- The API for adding and removing content filters.
- The rule format accepted by the installed WebKitGTK version.

Use the native content-filter mechanism as the baseline when available.

Build a dedicated ad-blocking component responsible for:
- Downloading filter lists over HTTPS.
- Validating downloaded content.
- Parsing or converting supported filter rules.
- Compiling rules into WebKit-compatible content filters.
- Installing compiled filters into relevant web views.
- Updating filter lists safely.
- Persisting filter configuration.
- Handling download, compilation and installation failures.
- Maintaining an optional allowlist.

Use maintained, reputable filter lists with licensing and attribution reviewed before distribution. Make list sources configurable.

Do not feed an arbitrary Adblock Plus or uBlock Origin ruleset directly into WebKit's compiler without verifying that its syntax is supported.

If conversion is necessary, implement and test a documented conversion layer. Preserve rule semantics where possible and report unsupported rules rather than silently treating them as effective.

## 3.2 Blocking requirements

The blocker should support, as far as the selected engine and rule format allow:
- Advertising requests.
- Known tracking requests.
- Third-party advertising resources.
- Common tracking scripts and pixels.
- Common advertising frames.
- Cosmetic hiding of advertising elements as an optional additional layer.

Network blocking must be the primary mechanism. CSS hiding alone is not an ad blocker.

Use an allowlist to disable blocking on a particular site. Match hostnames safely and avoid broad substring-based exceptions.

Handle redirects, subresources and newly created tabs correctly.

Ensure that filter updates do not leave web views in an inconsistent state. Apply updates atomically where possible and preserve the last known-good filter when a new update fails.

Do not claim that every advertisement can be blocked. First-party ads, server-side advertising and unsupported filter rules may not be blockable.

## 3.3 User interface

Add:
- A toolbar button showing blocker status.
- A per-site blocking toggle.
- A settings page for filter lists.
- Last successful update time.
- Download and compilation errors.
- A blocked-request counter if reliable instrumentation is available.
- A way to reset filter configuration.

Clearly distinguish the number of requests actually blocked from an estimate of advertising elements hidden.

## 3.4 Manifest V2 extension compatibility

In addition to native filtering, investigate whether the target WebKitGTK version supports the WebExtensions functionality required by Manifest V2.

Inspect the version-specific APIs and documentation, including WebKitWebExtension, WebKitWebView extension modes, extension loading, background execution, permissions, content scripts and extension lifecycle management where available.

The existence of a Manifest V2 mode does not prove that every Firefox or Chromium extension API is supported.

Implement an optional extension-loading subsystem only after documenting:
- The minimum required WebKitGTK version.
- Supported manifest fields.
- Supported permissions and APIs.
- Background-script limitations.
- Request interception and blocking limitations.
- Differences from Firefox's WebExtensions implementation.
- Unsupported features and appropriate user-facing errors.

Test a minimal Manifest V2 extension before attempting to load a complex ad blocker.

If the installed engine lacks a required API, do not fake compatibility. Keep native ad blocking functional, clearly report the limitation and implement an extension compatibility layer only where technically feasible.

Do not make the entire browser depend on Manifest V2 compatibility.

## 3.5 Ad-blocking acceptance tests

Create a local test website or controlled integration-test server that serves:
- A normal page.
- A known advertising resource.
- A known tracker resource.
- A blocked image or script.
- An allowed resource.
- A page that loads after a filter update.

Verify that:
1. Enabled rules block the intended network requests.
2. Allowlisted resources load.
3. Disabling the blocker allows previously blocked resources to load after the appropriate reload.
4. Updated rules take effect.
5. Invalid or unavailable lists do not crash the application.
6. Private browsing uses the expected filtering configuration.
7. The browser remains usable when filter lists cannot be downloaded.

# Phase 4 — Browser interface and GNOME integration

Use Firefox, Zen Browser and GNOME Web as functional references, not as interfaces to copy pixel-for-pixel.

Preserve Vingilot's identity and existing visual language.

Prioritize:
- A clear and compact navigation toolbar.
- A useful tab strip with close and new-tab controls.
- An accessible address and search bar.
- Bookmark controls and a bookmark manager.
- History and downloads views.
- A settings interface.
- Private-mode indicators.
- Responsive window sizing.
- Consistent context menus and dialogs.
- Light, dark and system themes where supported.
- Keyboard navigation, accelerators and tooltips.
- Clear loading, error and empty states.
- Consistent icons and spacing.
- Appropriate GNOME/GTK behavior.

Audit the existing layout before redesigning it. Prefer targeted improvements over replacing the entire interface.

Implement useful browser shortcuts, including:
- Ctrl+L: focus the address bar.
- Ctrl+T: new tab.
- Ctrl+W: close tab.
- Ctrl+Shift+T: reopen the last closed tab where supported.
- Ctrl+R: reload.
- Ctrl+D: bookmark the current page.
- Ctrl+H: history.
- Ctrl+J: downloads where supported.
- Ctrl+Shift+P: private browsing where appropriate for the platform and existing conventions.

Avoid shortcut conflicts with GTK, WebKit and text editing.

# Phase 5 — Security, reliability and maintainability

Audit:
- URL parsing and scheme handling.
- TLS and certificate errors.
- Download destination validation.
- File-opening behavior.
- Permission prompts.
- Untrusted web content.
- JavaScript bridges and privileged application actions.
- Extension permissions and untrusted extension installation.
- Settings and bookmark persistence.
- Resource cleanup and object lifetimes.
- Signal handlers and asynchronous callbacks.
- Memory leaks and crashes.
- Logging of private information.

Do not expose privileged browser functionality to ordinary websites through an unrestricted JavaScript bridge.

Keep native application actions separate from web content. Validate URLs, filesystem paths and external protocol handling.

Use secure defaults and explicit user consent for sensitive permissions.

# Phase 6 — Automated testing and final verification

Create or extend the project's existing test suite.

Include:
- Unit tests for bookmark and folder operations.
- Tests for rename, move and delete behavior.
- Tests for persistent data and migrations.
- Integration tests for tab creation and closure.
- Tests for normal and private storage isolation.
- Tests for history exclusion in private mode.
- Tests for filter compilation, installation and allowlisting.
- Tests for filter-list update failures.
- UI tests for context menus and folder management where feasible.
- A clean build and launch test.

Use the project's existing build system and testing framework. Do not introduce a new framework without justification.

For every phase:
1. Make a focused change.
2. Build the application.
3. Run relevant automated tests.
4. Test important behavior manually when automation is insufficient.
5. Fix regressions.
6. Summarize modified files and design decisions.
7. Record any remaining limitations honestly.

Do not mark tests as passed unless they were actually executed.

# Definition of done

Vingilot is ready for this milestone when:

- Existing features still work.
- Bookmark folders can be renamed and deleted safely.
- Bookmark and webpage context menus provide the expected actions.
- Tabs, history and downloads work consistently.
- Private browsing uses an isolated ephemeral session and does not leak into normal history.
- Native ad and tracker blocking demonstrably blocks matching network resources.
- The browser reports the real capabilities and limitations of Manifest V2 support.
- All critical tests pass.
- The project builds and launches in its documented target environment.
- Installation and usage instructions are updated.

# How to work

Start with the repository audit. Then implement the phases in order, prioritizing the essential browser functions, private browsing and reliable native ad blocking.

Do not ask me to manually implement code that you can implement directly in the repository.

Do not stop after presenting recommendations. Continue making changes, building, testing and fixing problems.

If an important architectural decision is required, explain the alternatives and recommend the least disruptive technically sound solution. If an API is unavailable, establish that through inspection or compilation rather than guessing.

At the end, provide:
- A summary of implemented features.
- A list of changed files.
- Build and test commands executed.
- Actual test results.
- Known limitations.
- Any migration or configuration steps.
- Remaining work, prioritized by importance.

Begin by auditing the existing Vingilot repository now.