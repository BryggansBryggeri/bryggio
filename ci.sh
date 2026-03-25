#!/usr/bin/env bash
set -euo pipefail

echo "=== Rust ==="
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test

echo ""
echo "=== Frontend ==="
cd bryggui
npm run ci

echo ""
echo "=== All checks passed ==="
