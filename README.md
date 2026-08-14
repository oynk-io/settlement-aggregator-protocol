# Oink Protocol

Production-oriented Soroban protocol scaffold for a two-sided settlement/liquidity network.

## Contracts
- `registry`: settler registration, capabilities, staking, reputation, slashing entrypoint.
- `payments`: payment lifecycle and settlement state machine.
- `treasury`: token custody, balances, locks, releases, admin withdrawal.
- `disputes`: dispute opening, evidence, resolution.
- `packages/oink-sdk`: shared contract types.

## Build
```bash
cargo test
stellar contract build
```

## Deploy
```bash
export SOURCE_ACCOUNT=<stellar-cli identity>
./scripts/deploy-testnet.sh
```

## Notes
Matching, FX pricing, KYC/AML, bank verification, and settlement routing are intentionally off-chain for v1. The contracts preserve verifiable on-chain state and events.
