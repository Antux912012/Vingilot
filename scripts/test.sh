#!/usr/bin/env bash
set -euo pipefail

export PATH="$HOME/.cargo/bin:$PATH"

echo "==> Running workspace tests..."
cargo test --workspace "$@"
