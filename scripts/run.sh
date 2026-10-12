#!/usr/bin/env bash
set -euo pipefail

export PATH="$HOME/.cargo/bin:$PATH"

echo "==> Running vingilot-ui binary..."
cargo run -p vingilot-ui -- "$@"
