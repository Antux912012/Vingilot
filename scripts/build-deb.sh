#!/usr/bin/env bash
set -euo pipefail

# Build script for Vingilot .deb package
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

VERSION="0.1.5"
REVISION="1"
ARCH="amd64"
PKG_NAME="vingilot"
DEB_NAME="${PKG_NAME}_${VERSION}_${ARCH}.deb"
DIST_DIR="${ROOT_DIR}/dist"
STAGE_DIR="${ROOT_DIR}/build/deb/${PKG_NAME}_${VERSION}_${ARCH}"

echo "==> Building release binaries with cargo..."
cargo build --release

echo "==> Preparing Debian package directory structure..."
rm -rf "$STAGE_DIR"
mkdir -p "${STAGE_DIR}/DEBIAN"
mkdir -p "${STAGE_DIR}/usr/bin"
mkdir -p "${STAGE_DIR}/usr/share/applications"
mkdir -p "${STAGE_DIR}/usr/share/metainfo"
mkdir -p "${STAGE_DIR}/usr/share/icons"
mkdir -p "$DIST_DIR"

echo "==> Installing binaries..."
install -Dm755 "${ROOT_DIR}/target/release/vingilot-ui" "${STAGE_DIR}/usr/bin/vingilot-ui"
install -Dm755 "${ROOT_DIR}/target/release/vingilot-cli" "${STAGE_DIR}/usr/bin/vingilot-cli"

echo "==> Installing desktop and metainfo files..."
install -Dm644 "${ROOT_DIR}/data/io.github.antux912012.Vingilot.desktop" "${STAGE_DIR}/usr/share/applications/io.github.antux912012.Vingilot.desktop"
install -Dm644 "${ROOT_DIR}/data/io.github.antux912012.Vingilot.metainfo.xml" "${STAGE_DIR}/usr/share/metainfo/io.github.antux912012.Vingilot.metainfo.xml"

echo "==> Installing icons..."
cp -r "${ROOT_DIR}/data/icons/hicolor" "${STAGE_DIR}/usr/share/icons/"
if [ -f "${ROOT_DIR}/data/icons/io.github.antux912012.Vingilot.svg" ]; then
    mkdir -p "${STAGE_DIR}/usr/share/icons/hicolor/scalable/apps"
    cp "${ROOT_DIR}/data/icons/io.github.antux912012.Vingilot.svg" "${STAGE_DIR}/usr/share/icons/hicolor/scalable/apps/"
fi
find "${STAGE_DIR}/usr/share/icons" -type f -exec chmod 644 {} +

# Calculate installed size in KB
INSTALLED_SIZE=$(du -sk "${STAGE_DIR}/usr" | awk '{print $1}')

echo "==> Generating DEBIAN/control..."
cat <<EOF > "${STAGE_DIR}/DEBIAN/control"
Package: ${PKG_NAME}
Version: ${VERSION}-${REVISION}
Section: web
Priority: optional
Architecture: ${ARCH}
Installed-Size: ${INSTALLED_SIZE}
Maintainer: Antonio <antux912012@github.com>
Depends: libc6, libglib2.0-0, libgtk-4-1, libadwaita-1-0, libwebkitgtk-6.0-4
Recommends: ca-certificates, gstreamer1.0-plugins-good, gstreamer1.0-plugins-bad
Homepage: https://github.com/Antux912012/Vingilot
Description: Native GTK4/Libadwaita Web Browser for Linux
 Vingilot is a lightweight, modern, privacy-respecting browser built
 with Rust, GTK4, Libadwaita, and WebKitGTK 6.0.
 Features include vertical tabs, Zen-style tab management, custom accents,
 native ad blocking, and AI assistant integration.
EOF

chmod 644 "${STAGE_DIR}/DEBIAN/control"
chmod 755 "${STAGE_DIR}/DEBIAN"

echo "==> Generating DEBIAN/postinst & postrm..."
cat <<'EOF' > "${STAGE_DIR}/DEBIAN/postinst"
#!/bin/sh
set -e
if [ "$1" = "configure" ]; then
    if which update-desktop-database >/dev/null 2>&1; then
        update-desktop-database -q /usr/share/applications || true
    fi
    if which gtk-update-icon-cache >/dev/null 2>&1; then
        gtk-update-icon-cache -q -t -f /usr/share/icons/hicolor || true
    fi
fi
exit 0
EOF
chmod 755 "${STAGE_DIR}/DEBIAN/postinst"

cat <<'EOF' > "${STAGE_DIR}/DEBIAN/postrm"
#!/bin/sh
set -e
if [ "$1" = "remove" ] || [ "$1" = "purge" ]; then
    if which update-desktop-database >/dev/null 2>&1; then
        update-desktop-database -q /usr/share/applications || true
    fi
    if which gtk-update-icon-cache >/dev/null 2>&1; then
        gtk-update-icon-cache -q -t -f /usr/share/icons/hicolor || true
    fi
fi
exit 0
EOF
chmod 755 "${STAGE_DIR}/DEBIAN/postrm"

echo "==> Building .deb package using dpkg-deb..."
dpkg-deb --build --root-owner-group "$STAGE_DIR" "${DIST_DIR}/${DEB_NAME}"
ln -sf "${DEB_NAME}" "${DIST_DIR}/${PKG_NAME}_${VERSION}-${REVISION}_${ARCH}.deb"

echo "==> Debian package created successfully: ${DIST_DIR}/${DEB_NAME}"
ls -lh "${DIST_DIR}/${DEB_NAME}"
