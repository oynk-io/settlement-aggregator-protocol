# Oynk Settlement Protocol

Soroban MVP for Oynk's two-sided settlement and liquidity network.

## Maintainer

The contract implementation is maintained by
[Emmanuel Ekoja (`emmanuelekoja`)](https://github.com/emmanuelekoja). Published
contract commits use the verified project identity
`emmanuelekoja <codeevangelist@gmail.com>`.

## Active workspace

- `contracts/settlement`: settlement lifecycle, escrow, claims, refunds, and disputes.
- `packages/oynk-sdk`: shared settlement types, errors, and events.

The `registry`, `treasury`, and `disputes` contract directories are design drafts.
They are intentionally excluded from the Cargo workspace and are not built or
deployed by the repository scripts.

## Build
```bash
./scripts/build.sh
```

The build runs SDK invariant tests, produces the settlement WASM, and executes
WASM-level fiat-to-crypto and fiat-to-fiat lifecycle tests.

The test suites are public in
[`packages/oynk-sdk/src/test.rs`](packages/oynk-sdk/src/test.rs) and
[`integration-tests/src/lib.rs`](integration-tests/src/lib.rs).

## Published build evidence

[`deployments/candidate.json`](deployments/candidate.json) records the exact
source commit, locked toolchain, generated WASM hash, and passing test counts
for the current public contract version. The recorded source commit builds to
`d16b5f8a2b9971e2ea45bf0737731ed6348bd83b7c30fde275589f69ea132bc9`.

## Deploy
```bash
export SOURCE_ACCOUNT=<stellar-cli identity>
./scripts/deploy-testnet.sh
```

The deployment script builds and deploys only
`oynk-settlement-protocol-contract`. It defaults to testnet; set `NETWORK`
explicitly for any other configured Stellar network.

## Mainnet deployment evidence

[`deployments/mainnet.json`](deployments/mainnet.json) records the publicly
observable contract ID, original creation evidence, constructor addresses,
source commit, current WASM hash, and the September 3, 2026 upload and upgrade
transactions. The contract now runs the reproducible artifact recorded in
`deployments/candidate.json`. The original July 2026 WASM remains identified
separately as source-unverified historical provenance.

## Notes
Matching, FX pricing, KYC/AML, bank verification, and settlement routing are intentionally off-chain for v1. The contracts preserve verifiable on-chain state and events.
