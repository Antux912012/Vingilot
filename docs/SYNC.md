# Vingilot Local Sync Guide

Vingilot uses portable `.vingilot` bundle archives for peer-to-peer and offline machine-to-machine synchronization.

---

## 1. Sync Principles

- **No Cloud Account Required:** Sync happens directly between devices via file transfer (USB drive, local network, Syncthing, SSH).
- **Security by Default:** Passwords, active cookies, sessions, and payment info are strictly excluded from normal bundles.
- **Passphrase Encryption:** Bundles can be encrypted using AES-256-GCM with key stretching derived from a user passphrase.

---

## 2. Using the Export Wizard

1. Open **Settings** or the **Sync Menu**.
2. Launch the **Export Wizard**.
3. Choose the data categories to bundle:
   - Bookmarks
   - Browsing History
   - Search Engines
   - User Settings
   - Reading List
   - Userscripts
   - Círdan Encrypted Credentials *(Optional opt-in)*
4. Set an optional encryption passphrase.
5. Save your `.vingilot` archive.

---

## 3. Using the Import Wizard

1. Launch the **Import Wizard** on your new device.
2. Select your `.vingilot` file.
3. If encrypted, provide your passphrase to unlock.
4. Select your import mode:
   - **Merge:** Intelligently combines bookmarks, history, and reading list entries.
   - **Replace:** Overwrites existing profile data with incoming bundle data.
   - **New Profile:** Loads the bundle into an isolated new profile.
5. An automatic backup is created before any destructive operation.

---

## 4. Terminal CLI Sync

You can also export and import bundles directly from your terminal:
```bash
# Export encrypted bundle
vingilot-cli export --output ~/backup.vingilot --passphrase "my-secure-key"

# Import bundle
vingilot-cli import ~/backup.vingilot --passphrase "my-secure-key"
```
