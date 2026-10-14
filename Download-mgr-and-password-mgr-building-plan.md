# Vingilot Browser — Phase 7 and Phase 8 Engineering Specification

## Project

Repository: https://github.com/Antux912012/Vingilot

Implement two new major features in the existing Vingilot desktop browser:

1. A complete Download Manager with BitTorrent and magnet-link support.
2. A secure, local Password Manager with encrypted storage, password-protected export and import, and browser credential autofill.

You are the developer who built the existing project. Continue working on the current repository. Do not create a separate browser, rewrite the application or replace working components unnecessarily.

The existing browser roadmap includes bookmarks, context menus, private browsing, WebKitGTK 6.0 compatibility and native ad blocking. Preserve these features and integrate the new functionality with them.

## General engineering rules

Before coding:
- Inspect the current source tree and architecture.
- Identify the programming language, UI toolkit, WebKitGTK version, persistence mechanisms and dependency manager.
- Inspect existing downloads, browser settings, tab management, private browsing and security-related code.
- Reuse existing UI conventions and persistence patterns where appropriate.
- Verify the availability and compatibility of every external library and API.
- Identify the project's actual build and test commands.

Do not invent source files, API functions or test results.

Implement the work in small, independently testable milestones. Build the project and run relevant tests after every milestone. Fix regressions before moving on.

Do not leave placeholder buttons, simulated transfer progress, fake encryption, nonfunctional import/export or unfinished event handlers.

If a technical requirement cannot be met, explain the actual limitation and implement the safest supported alternative.

# PHASE 7 — COMPLETE DOWNLOAD MANAGER

## 7.1 Architecture

Build a dedicated download-management subsystem integrated into the existing browser.

Separate:
- Download task model.
- Download queue and scheduling.
- Ordinary HTTP/HTTPS transfer handling.
- BitTorrent transfer handling.
- Persistent task metadata.
- Download destination management.
- UI and user actions.
- Notifications and error reporting.

Use the existing WebKitGTK download APIs for downloads initiated by webpage navigation or WebKit download requests.

Investigate the installed WebKitGTK 6.0 APIs and use the supported download lifecycle signals and methods. Do not assume that an older WebKitGTK API is identical to the installed version.

For BitTorrent, evaluate libtorrent-rasterbar or another maintained, suitable native library. Verify its version, license, build dependencies, Linux packaging availability, language bindings and integration cost before selecting it.

Keep the torrent engine behind a well-defined interface so it does not contaminate the ordinary download implementation.

If a native dependency is unavailable, do not silently substitute a fake implementation.

## 7.2 Unified download task model

Create a persistent task model with appropriate fields for:
- Unique task ID.
- Task type: ordinary download, torrent file or magnet.
- Original URI or magnet URI, where appropriate.
- Display name.
- Destination path.
- Total size, if known.
- Bytes received.
- Transfer speed.
- Estimated remaining time, if calculable.
- Current state.
- Creation and completion timestamps.
- Error information.
- Resume metadata where supported.
- Torrent-specific metadata, where applicable.

Represent states explicitly, for example:
- Queued.
- Downloading.
- Paused.
- Verifying.
- Completed.
- Failed.
- Cancelled.
- Seeding, for torrents.

Define valid state transitions. Do not report a cancelled task as completed or display a failed transfer as successful.

Do not persist passwords, HTTP authorization headers, private browsing URLs or other unnecessary sensitive request data in download metadata.

## 7.3 Ordinary download functionality

Implement:
- Automatic interception of WebKit downloads.
- Download destination selection.
- A configurable default download directory.
- Collision detection and safe filename handling.
- Progress reporting.
- Current speed and estimated remaining time where reliable.
- Pause and resume when supported by the transfer mechanism.
- Cancellation.
- Retry after appropriate failures.
- Opening completed files using the desktop's normal file handler.
- Showing the downloaded file in its containing directory.
- Removal of entries from the visible list.
- Optional deletion of completed files, with confirmation.
- Failure reporting and recovery.
- Configurable concurrent-download limits.

Never overwrite an existing file without an explicit policy and appropriate user consent.

When a transfer cannot resume, explain the limitation and restart safely if the user requests it.

Use actual transfer callbacks and byte counts rather than simulated progress.

## 7.4 Torrent and magnet support

The browser must support:
- Opening magnet: links from the address bar.
- Opening magnet links clicked on webpages.
- Opening local .torrent files.
- Adding a torrent through the download manager UI.
- Inspecting torrent name, files and total size before downloading.
- Selecting which files to download when the engine supports it.
- Selecting the download destination.
- Starting, pausing, resuming and removing torrents.
- Displaying download and upload speeds.
- Displaying downloaded and uploaded bytes.
- Showing connected peers and seeders where available.
- Showing tracker status and errors where available.
- Reporting piece verification and completion status.
- Configuring a seeding ratio or seeding time limit.
- Stopping seeding independently of deleting the downloaded files.
- Supporting DHT, peer exchange and tracker communication according to the engine's capabilities and user settings.

Use the torrent library's actual APIs and state reporting.

Validate magnet URIs and reject malformed or unsupported input gracefully.

Handle incomplete metadata retrieval, unavailable trackers, stalled peers, network failures and corrupt torrent data.

Do not block the GTK main thread with network operations or torrent metadata processing.

Implement a dedicated worker, event-driven integration or other architecture appropriate to the selected library.

If the torrent engine supports session-state persistence, investigate how to restore active transfers safely after an application restart.

Do not assume that WebKitGTK can itself download BitTorrent content. The dedicated torrent engine must own torrent networking.

## 7.5 Torrent security and privacy

Provide settings for:
- Maximum concurrent torrents.
- Maximum concurrent downloads.
- Download and upload speed limits.
- Seeding behavior.
- DHT and peer-discovery features.
- Listening port configuration where supported.
- Proxy configuration only where the engine genuinely supports the selected proxy mode.

Avoid leaking private browsing state into ordinary download history.

Do not automatically send torrent traffic through a proxy or VPN unless the configuration actually supports it. Clearly explain any traffic that bypasses the configured proxy.

Treat torrent names, metadata, trackers, file paths and downloaded content as untrusted input.

Prevent path traversal and unsafe extraction paths. Do not automatically extract downloaded archives.

Never claim that torrent activity is anonymous.

## 7.6 Download manager UI

Create a consistent interface using the existing Vingilot UI.

Provide:
- A download-manager window, page or sidebar.
- A toolbar button and useful keyboard shortcut.
- A unified list for ordinary downloads and torrents.
- Clear task icons and status indicators.
- Progress bars and meaningful transfer information.
- Pause, resume, retry, cancel and remove actions.
- A details view for torrents.
- Search and status filtering.
- Context menus for download items.
- Empty, loading, error and completed states.
- Notifications for completion and failures, where supported.

Avoid showing irrelevant torrent controls for ordinary files.

Do not block the interface when a transfer starts, stops or updates.

## 7.7 Persistence and recovery

Persist task history and relevant resume data safely.

On startup:
- Reconcile stored tasks with files on disk.
- Detect incomplete or missing files.
- Restore ordinary downloads only when the transfer mechanism supports recovery.
- Restore torrent sessions using the engine's supported resume mechanism.
- Handle stale, invalid or incompatible resume data.
- Avoid automatically starting dangerous or unexpected transfers without appropriate settings and user expectations.

Do not mark incomplete files as complete simply because they exist.

Use transactions or atomic updates for persistent task state where appropriate.

## 7.8 Download testing

Create tests for:
- Successful local HTTP downloads.
- Failed and interrupted transfers.
- Cancellation.
- Pause/resume where supported.
- Filename collisions.
- Invalid destination paths.
- Missing and deleted files.
- Task restoration after restart.
- Magnet URI parsing.
- Torrent metadata retrieval.
- Torrent pause and resume.
- Piece verification and completion.
- Corrupt or invalid torrent files.
- UI responsiveness during active transfers.
- Private-session isolation.

Prefer a local controlled test server and test torrents rather than relying exclusively on external networks.

Do not report successful torrent tests unless actual transfer and verification occurred.

# PHASE 8 — SECURE LOCAL PASSWORD MANAGER

## 8.1 Threat model and architecture

Build a local password manager that protects credentials at rest and supports secure backup and migration.

First document the threat model, including:
- A stolen or copied vault file.
- An attacker attempting offline master-password guesses.
- A malicious or malformed import file.
- Accidental exposure through logs, clipboard or crash reporting.
- An attacker who gains access to the unlocked application process.

Use established cryptographic libraries and APIs.

Never implement a custom encryption algorithm, key derivation algorithm or cryptographic random-number generator.

Keep password management in a dedicated subsystem. Do not expose unrestricted access to the vault to webpage JavaScript.

## 8.2 Encrypted vault storage

Store the password vault locally.

Evaluate a database or another suitable storage mechanism based on the existing architecture. Whatever the storage format, credential contents must be encrypted at rest.

Requirements:
- Use a maintained cryptographic implementation.
- Derive a cryptographic key from the master password using Argon2id where available.
- Use a fresh cryptographically secure salt.
- Use authenticated encryption, preferably AES-256-GCM or another established, supported AEAD scheme.
- Generate nonces using the cryptographic library's secure random facilities.
- Enforce nonce uniqueness according to the chosen algorithm's requirements.
- Authenticate the relevant format metadata and ciphertext.
- Validate stored algorithm parameters before performing expensive operations.
- Apply reasonable limits to KDF memory, iterations and input sizes.
- Handle incorrect master passwords and corrupted vaults safely.

Do not use a fast general-purpose hash such as SHA-256 as the sole master-password key derivation function.

Do not store the master password, plaintext credentials or derived encryption key in persistent configuration files, logs or debug output.

Avoid keeping plaintext credentials in memory longer than necessary. Clear sensitive buffers when practical, recognizing that memory zeroization cannot guarantee removal of every runtime copy.

## 8.3 Vault initialization and unlocking

Implement:
- First-run vault creation.
- Master-password confirmation.
- Master-password strength guidance.
- Unlocking an existing vault.
- Clear feedback for incorrect passwords.
- Manual vault locking.
- Configurable inactivity locking.
- Locking when the browser closes, according to the selected policy.
- Safe handling of multiple windows and concurrent vault access.

Use the operating system's secure credential storage only for optional convenience features, such as a user-authorized session key, if appropriate. Do not silently bypass the master password or persist an unprotected vault key.

Explain that losing the master password may make the encrypted vault unrecoverable.

## 8.4 Credential entry model

Support records containing:
- Entry ID.
- Website URL or origin.
- Username.
- Password.
- Optional display title.
- Optional notes.
- Creation and modification timestamps.
- Optional tags or folders.

Implement:
- Create.
- View.
- Edit.
- Delete.
- Search.
- Sort.
- Copy username.
- Copy password.
- Generate password.
- Detect possible duplicate entries.
- Handle missing or malformed URLs.
- Optional notes and metadata.

Validate all imported entries. Preserve stable IDs where appropriate and resolve duplicate IDs safely.

Never log passwords, decrypted vault contents or master-password input.

## 8.5 Password generator

Provide a configurable password generator with:
- Adjustable length.
- Uppercase letters.
- Lowercase letters.
- Digits.
- Symbols.
- Optional exclusion of ambiguous characters.

Use a cryptographically secure random generator.

Avoid modulo bias when selecting characters. Use a maintained library or a correct rejection-sampling implementation.

Display the generated password clearly and allow the user to copy it or use it when creating a credential.

Do not claim a password is strong merely because it is long. Explain relevant generator settings where useful.

## 8.6 Website password saving

Implement an opt-in password-saving flow for supported login forms.

Investigate the actual WebKitGTK 6.0 APIs and determine the safest practical way to detect login submissions and identify username and password fields.

Use narrowly scoped content scripts or another suitable integration where required. Do not expose the vault's decryption key or general-purpose privileged methods to webpage JavaScript.

When a user submits a login form:
- Detect candidate credentials only where supported.
- Ask whether to save a new login.
- Offer to update an existing credential.
- Allow the user to decline.
- Respect site-specific exclusions.
- Avoid saving credentials from private browsing by default.
- Never save credentials silently without the configured consent policy.

Handle password managers and login forms that use dynamic JavaScript, redirects or multiple steps as far as practical.

Document unsupported cases honestly.

## 8.7 Credential autofill

Implement secure, user-controlled autofill.

Requirements:
- Match entries against the actual page origin.
- Use correct URL parsing and origin comparison.
- Do not match by arbitrary substring.
- Handle HTTP and HTTPS carefully.
- Do not autofill credentials into unrelated origins.
- Require appropriate user interaction or explicit preferences.
- Support manual selection when multiple logins exist.
- Allow autofill to be disabled globally or per site.
- Avoid automatically filling credentials in private browsing unless explicitly enabled by the user.
- Prevent webpage scripts from directly reading vault records.
- Keep privileged operations outside untrusted page contexts.

Prefer explicit user activation for password filling initially. Consider broader automation only after the security implications have been reviewed.

Test normal pages, HTTPS pages, subdomains, lookalike domains, redirects and dynamically generated login forms.

## 8.8 Password-protected encrypted export

Implement an encrypted export format specifically designed for Vingilot.

The export must contain:
- A format identifier.
- A format version.
- The encryption algorithm identifier.
- The KDF identifier and parameters.
- A random salt.
- A fresh nonce or other algorithm-appropriate nonce data.
- Authenticated metadata.
- The encrypted vault payload.

The plaintext payload should contain the required credential records and supported metadata in a documented, versioned serialization format.

Use Argon2id and authenticated encryption as described above, unless the actual platform constraints require a documented, secure alternative.

The export password must be provided explicitly by the user and confirmed before exporting.

Do not reuse the master password automatically unless the user explicitly chooses to enter the same password.

Do not include a plaintext copy of the vault or a plaintext password field in the export.

Write the export safely:
- Use secure temporary-file handling where needed.
- Avoid partial exports being mistaken for completed backups.
- Use atomic file replacement where appropriate.
- Prevent unintended overwrites.
- Handle interrupted writes.
- Avoid leaving plaintext temporary files behind.

The resulting export must be portable between supported Vingilot installations and independent of the original machine's encryption key.

Do not bind the encrypted backup exclusively to a machine-specific secret if portability is required.

## 8.9 Password-protected import

Implement import of Vingilot's encrypted export format.

The import flow must:
1. Let the user select an export file.
2. Validate the file structure and format version.
3. Reject malformed or excessively large inputs safely.
4. Request the export password.
5. Validate KDF parameters before deriving a key.
6. Authenticate and decrypt the payload.
7. Validate the decrypted structure and every credential record.
8. Display a preview and import summary.
9. Let the user choose how duplicate records are handled.
10. Commit changes only after successful validation and user confirmation.

An incorrect password or modified ciphertext must produce a clear error and must not partially import records.

Never treat successful decryption alone as proof that every imported record is valid.

Implement safe duplicate-handling options, such as:
- Skip duplicates.
- Update matching records.
- Import as separate entries.

Use an explicit, documented matching policy.

Preserve the existing vault if any stage fails.

## 8.10 Import from other password managers

Support common browser password exports where practical, especially CSV files from browsers that users commonly migrate from.

Requirements:
- Identify supported formats explicitly.
- Detect headers rather than assuming a fixed column order.
- Validate encoding and field sizes.
- Handle quoted fields, commas, embedded newlines and empty values correctly.
- Show a preview before committing.
- Report skipped and invalid records.
- Avoid silently discarding credentials.
- Never log imported passwords.

Plaintext CSV is inherently sensitive. Warn the user that the source file contains unencrypted credentials, ask for confirmation, and recommend removing the plaintext file after verifying the import.

Do not misrepresent plaintext CSV as encrypted or secure.

If other password-manager formats are unsupported, document them and design the importer so additional formats can be added later.

## 8.11 Password-manager UI

Provide:
- A password-manager window or dedicated view.
- A clear locked/unlocked state.
- A master-password unlock screen.
- A searchable credential list.
- An entry detail and edit view.
- Create and delete actions.
- A password generator.
- Copy username and password actions.
- Export and import actions.
- A lock-now control.
- Security and autofill preferences.
- Clear error, empty and loading states.

Mask passwords by default. Reveal them only through an explicit user action.

Use the existing browser UI style and accessibility conventions.

## 8.12 Clipboard and session security

When credentials are copied:
- Copy only after an explicit user action.
- Never log clipboard contents.
- Offer an optional clipboard-clearing timeout.
- Clear the clipboard only if it still contains the value Vingilot copied, so it does not destroy a newer clipboard entry from another application.
- Clearly explain the limitations of clipboard clearing.

Do not claim that clipboard contents can always be erased from clipboard history managers or other applications.

Lock the vault according to the configured policy and ensure that the locked state disables credential access and autofill.

## 8.13 Password-manager testing

Implement tests for:
- Vault initialization and unlocking.
- Incorrect master password.
- Correct encryption and decryption.
- Salt and nonce generation.
- Authentication failure after ciphertext modification.
- Corrupted and unsupported export versions.
- Excessive or malicious KDF parameters.
- Large and malformed input files.
- Duplicate handling.
- Atomic import and rollback on failure.
- Export and import round trips.
- Recovery after application restart.
- Password generator behavior.
- Origin matching.
- Autofill restrictions.
- Private-session behavior.
- Logging and error-message redaction.

Include regression tests ensuring that existing bookmarks, history, settings, downloads and private browsing remain functional.

Do not use production credentials in tests. Use synthetic test accounts.

# PHASE 9 — INTEGRATION AND RELEASE READINESS

Integrate both features with Vingilot's existing architecture.

Ensure:
- The download manager and password manager use consistent UI patterns.
- Private browsing follows explicit, documented policies.
- The ad blocker does not interfere with legitimate password forms or download flows without a clear reason.
- Settings are persisted and migrated safely.
- Background work does not block the UI.
- Application shutdown cleans up active workers and sensitive resources.
- The project builds using its documented dependencies.
- New dependencies and licenses are documented.
- The README explains torrent support, download recovery, vault setup, encryption, export/import and limitations.

# FINAL ACCEPTANCE CRITERIA

The work is complete only when:

1. Ordinary browser downloads work through the real WebKitGTK download implementation.
2. Magnet links and .torrent files launch real torrent tasks.
3. Torrent transfers can be paused, resumed, monitored and verified.
4. Download tasks and supported resume state survive an application restart.
5. The password vault stores credentials encrypted at rest.
6. The vault can be locked and unlocked using the master password.
7. Credentials can be created, edited, searched, copied and deleted.
8. Password-protected exports can be imported into another supported Vingilot installation.
9. Incorrect export passwords and tampered exports are rejected.
10. Supported browser CSV exports can be imported with explicit plaintext-file warnings.
11. Autofill is origin-restricted and controlled by the user.
12. Private browsing does not accidentally save private history or credentials.
13. Automated tests pass, and any untested behavior is clearly identified.
14. The application builds and launches in the documented target environment.

# EXECUTION INSTRUCTIONS

Start by auditing the existing repository and identifying the best integration points.

Then implement Phase 7 in small increments, building and testing after each increment.

After the Download Manager is stable, implement Phase 8 in small increments, prioritizing the encryption format, vault correctness, import/export and security tests before advanced autofill behavior.

Finally, perform integration and regression testing.

Do not stop after proposing an architecture. Implement the features directly in the repository wherever possible.

At the end, report:
- Features implemented.
- Files changed.
- Dependencies added and their licenses.
- Build and test commands actually executed.
- Actual test results.
- Security design and limitations.
- Data migration and backup instructions.
- Any remaining work.

Never claim encryption, privacy, torrent support or tests work unless the corresponding implementation has been verified.