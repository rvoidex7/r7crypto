#!/usr/bin/env bash
# Full local quality gate. Mirrors the CI checks, including the wasm-specific
# guard against host-only APIs that compile but panic in the browser.
set -euo pipefail

cd "$(dirname "$0")/.."
export PATH="$HOME/.cargo/bin:$PATH"

echo "== fmt =="
cargo fmt --all -- --check

echo "== clippy (native, all features) =="
cargo clippy --all-targets --all-features --locked -- -D warnings

echo "== tests =="
cargo test --all-features --locked

echo "== feature matrix =="
cargo check --all-targets --all-features --locked
cargo check --all-targets --no-default-features --features gui --locked
cargo check --all-targets --no-default-features --locked

echo "== wasm type-check =="
cargo check --lib --bins --target wasm32-unknown-unknown --all-features --locked
cargo check --lib --target wasm32-unknown-unknown --no-default-features --features gui --locked

echo "== wasm host-only API guard =="
CLIPPY_CONF_DIR=ci/wasm-clippy \
  cargo clippy --lib --target wasm32-unknown-unknown --no-default-features --features gui --locked -- -D warnings -D clippy::disallowed_methods

echo "All checks passed."
