#!/usr/bin/env bash
set -euo pipefail

export PATH="$HOME/.cargo/bin:$PATH"

echo "==> Running cargo fmt check..."
cargo fmt --all --check

echo "==> Running cargo clippy..."
cargo clippy --all-targets --all-features -- -D warnings

echo "==> Running cargo test..."
cargo test --workspace

echo "==> All quality checks passed!"
