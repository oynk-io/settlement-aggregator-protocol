#!/usr/bin/env bash
set -euo pipefail
: "${SOURCE_ACCOUNT:?Set SOURCE_ACCOUNT}"
: "${NETWORK:=testnet}"
stellar contract build
for C in registry payments treasury disputes; do
  WASM="target/wasm32v1-none/release/oink_${C}.wasm"
  echo "Deploying $C from $WASM"
  stellar contract deploy --wasm "$WASM" --source "$SOURCE_ACCOUNT" --network "$NETWORK"
done
