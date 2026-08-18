#!/usr/bin/env bash
set -euo pipefail
cargo test --workspace --locked
stellar contract build --package oynk-settlement-protocol-contract
cargo test --manifest-path integration-tests/Cargo.toml --locked
