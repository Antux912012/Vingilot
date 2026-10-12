# Vingilot Browser Import Guide

Vingilot makes migrating from existing browsers seamless and safe.

---

## 1. Supported Browsers & Formats

- **Mozilla Firefox:**
  - Direct import from `places.sqlite` (bookmarks and history).
  - JSON backup files (`bookmarks-YYYY-MM-DD.json`).
  - HTML bookmark exports.
- **Chromium-based Browsers:**
  - Google Chrome, Chromium, Brave, Microsoft Edge, and Vivaldi.
  - JSON `Bookmarks` files.
  - SQLite `History` databases.
- **Generic Formats:**
  - Standard Netscape Bookmark HTML (`.html`).

---

## 2. Safety & Non-destructive Operation

- Vingilot **never** writes to or modifies external browser directories.
- SQLite databases are copied to isolated temporary locations before reading to prevent lock contention with currently running browsers.
- Passwords and active session cookies are **not** imported to prevent accidental credential leakage.

---

## 3. Importing via Graphical Wizard

1. Open the **Browser Import Dialog** from the menu or settings.
2. The dialog automatically lists detected local profiles (`Firefox`, `Chrome`, `Chromium`, `Brave`, `Edge`, `Vivaldi`).
3. Or choose **Manual File Selection** to pick an HTML bookmark export.
4. Check **Import Bookmarks** and/or **Import History**.
5. Click **Import** to merge records into your active profile.

---

## 4. Importing via Terminal CLI

```bash
# Import Firefox bookmarks and history
vingilot-cli import-firefox ~/.mozilla/firefox/*.default-release/places.sqlite

# Import Google Chrome bookmarks
vingilot-cli import-chrome ~/.config/google-chrome/Default/Bookmarks

# Import standard Netscape HTML bookmarks
vingilot-cli import-bookmarks-html ~/Downloads/bookmarks.html
```
