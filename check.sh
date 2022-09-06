#!/usr/bin/env bash
# This scripts runs various CI-like checks in a convenient way.
set -eux

cargo check --workspace --all-features --all-targets
cargo clippy --workspace --all-features --all-targets -- -D warnings -W clippy::all
cargo test --workspace --all-features --all-targets
cargo test --workspace --all-features --doc
cargo fmt --all -- --check

cargo check --workspace --all-features --all-targets --target wasm32-unknown-unknown
cargo clippy --workspace --all-features --all-targets --target wasm32-unknown-unknown -- -D warnings -W clippy::all
trunk build
