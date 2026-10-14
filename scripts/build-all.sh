#!/usr/bin/env bash
set -euo pipefail

# Build script to generate all non-Flatpak packages (.deb and .AppImage)
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

echo "=========================================="
echo " Building Vingilot Packages (.deb & .AppImage)"
echo "=========================================="

"${ROOT_DIR}/scripts/build-deb.sh"
echo ""
"${ROOT_DIR}/scripts/build-appimage.sh"
echo ""

echo "=========================================="
echo " Build Completed Successfully!"
echo " Output files in dist/:"
echo "=========================================="
ls -lh "${ROOT_DIR}/dist"
