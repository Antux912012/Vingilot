#!/usr/bin/env bash
set -euo pipefail

# Build script for Vingilot AppImage package
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

VERSION="0.1.5"
ARCH="x86_64"
DIST_DIR="${ROOT_DIR}/dist"
APPDIR="${ROOT_DIR}/build/appimage/Vingilot.AppDir"
OUTPUT_APPIMAGE="${DIST_DIR}/Vingilot-${VERSION}-${ARCH}.AppImage"
TOOLS_DIR="${ROOT_DIR}/build/tools"
APPIMAGETOOL="${TOOLS_DIR}/appimagetool"

echo "==> Building release binaries with cargo..."
cargo build --release

echo "==> Preparing AppDir directory structure..."
rm -rf "$APPDIR"
mkdir -p "${APPDIR}/usr/bin"
mkdir -p "${APPDIR}/usr/share/applications"
mkdir -p "${APPDIR}/usr/share/metainfo"
mkdir -p "${APPDIR}/usr/share/icons/hicolor"
mkdir -p "$DIST_DIR"

echo "==> Installing binaries..."
install -Dm755 "${ROOT_DIR}/target/release/vingilot-ui" "${APPDIR}/usr/bin/vingilot-ui"
install -Dm755 "${ROOT_DIR}/target/release/vingilot-cli" "${APPDIR}/usr/bin/vingilot-cli"

echo "==> Installing desktop and metainfo files..."
install -Dm644 "${ROOT_DIR}/data/io.github.antux912012.Vingilot.desktop" "${APPDIR}/usr/share/applications/io.github.antux912012.Vingilot.desktop"
install -Dm644 "${ROOT_DIR}/data/io.github.antux912012.Vingilot.metainfo.xml" "${APPDIR}/usr/share/metainfo/io.github.antux912012.Vingilot.metainfo.xml"
install -Dm644 "${ROOT_DIR}/data/io.github.antux912012.Vingilot.metainfo.xml" "${APPDIR}/usr/share/metainfo/io.github.antux912012.Vingilot.appdata.xml"

# Root-level desktop file and icons required by AppImage specification
cp "${ROOT_DIR}/data/io.github.antux912012.Vingilot.desktop" "${APPDIR}/"
cp "${ROOT_DIR}/data/icons/hicolor/256x256/apps/io.github.antux912012.Vingilot.png" "${APPDIR}/io.github.antux912012.Vingilot.png"
if [ -f "${ROOT_DIR}/data/icons/io.github.antux912012.Vingilot.svg" ]; then
    cp "${ROOT_DIR}/data/icons/io.github.antux912012.Vingilot.svg" "${APPDIR}/"
fi
ln -sf "io.github.antux912012.Vingilot.png" "${APPDIR}/.DirIcon"

echo "==> Installing hicolor icons..."
cp -r "${ROOT_DIR}/data/icons/hicolor/"* "${APPDIR}/usr/share/icons/hicolor/"
if [ -f "${ROOT_DIR}/data/icons/io.github.antux912012.Vingilot.svg" ]; then
    mkdir -p "${APPDIR}/usr/share/icons/hicolor/scalable/apps"
    cp "${ROOT_DIR}/data/icons/io.github.antux912012.Vingilot.svg" "${APPDIR}/usr/share/icons/hicolor/scalable/apps/"
fi

echo "==> Creating AppRun entry point..."
cat <<'EOF' > "${APPDIR}/AppRun"
#!/bin/sh
set -e

# AppRun launcher for Vingilot
HERE="$(dirname "$(readlink -f "$0")")"

export PATH="${HERE}/usr/bin:${PATH}"
export LD_LIBRARY_PATH="${HERE}/usr/lib:${HERE}/usr/lib/x86_64-linux-gnu:${LD_LIBRARY_PATH:-}"
export XDG_DATA_DIRS="${HERE}/usr/share:${XDG_DATA_DIRS:-/usr/local/share:/usr/share}"

# Ensure GTK GL renderer is selected to prevent popover black box artifacts
export GSK_RENDERER="${GSK_RENDERER:-gl}"

# If called as CLI or with CLI flags, route to appropriate binary
if [ "$1" = "--cli" ]; then
    shift
    exec "${HERE}/usr/bin/vingilot-cli" "$@"
fi

exec "${HERE}/usr/bin/vingilot-ui" "$@"
EOF
chmod +x "${APPDIR}/AppRun"

echo "==> Ensuring appimagetool is available..."
if ! command -v appimagetool >/dev/null 2>&1; then
    if [ ! -f "$APPIMAGETOOL" ]; then
        mkdir -p "$TOOLS_DIR"
        echo "==> Downloading appimagetool..."
        curl -fsSL -o "$APPIMAGETOOL" https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage
        chmod +x "$APPIMAGETOOL"
    fi
    TOOL_CMD="$APPIMAGETOOL"
else
    TOOL_CMD="$(command -v appimagetool)"
fi

echo "==> Packaging AppImage with appimagetool..."
export ARCH="x86_64"

# Attempt standard run; fall back to --appimage-extract-and-run if FUSE is not enabled
if "$TOOL_CMD" --version >/dev/null 2>&1; then
    "$TOOL_CMD" --no-appstream "$APPDIR" "$OUTPUT_APPIMAGE"
else
    "$TOOL_CMD" --appimage-extract-and-run --no-appstream "$APPDIR" "$OUTPUT_APPIMAGE"
fi

chmod +x "$OUTPUT_APPIMAGE"
echo "==> AppImage created successfully: $OUTPUT_APPIMAGE"
ls -lh "$OUTPUT_APPIMAGE"
