#!/usr/bin/env bash
set -euo pipefail
: "${SOURCE_ACCOUNT:?Set SOURCE_ACCOUNT}"
: "${NETWORK:=testnet}"
stellar contract build --package oynk-settlement-protocol-contract

WASM="target/wasm32v1-none/release/oynk_settlement_protocol_contract.wasm"

stellar contract deploy \
  --wasm "$WASM" \
  --source "$SOURCE_ACCOUNT" \
  --network "$NETWORK"
