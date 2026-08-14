#!/usr/bin/env bash
set -euo pipefail
cargo test
stellar contract build
