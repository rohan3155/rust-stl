#!/usr/bin/env bash
# Shared validation entry point for rust-stl.
#
# Run from the repository root. Exits immediately if any required check fails.
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

cargo fmt --all -- --check
cargo test --all-targets --all-features
cargo test --doc --all-features
cargo clippy --all-targets --all-features -- -D warnings
cargo doc --no-deps --all-features
cargo package --locked
