# Oink Protocol Architecture

```mermaid
flowchart LR
  Customer[Customer pays local fiat/crypto] --> Backend[Oink Backend Matching Engine]
  Backend --> Settlement[Settlement Contract]
  Settler[Settler liquidity actor] --> Settlement
  Settlement --> Escrow[Stellar asset escrow]
  Indexer[Event Indexer] --> Dashboard[Analytics Dashboard]
  Settlement --> Indexer
```

Only the settlement contract and shared SDK are active Cargo workspace members.
Registry, treasury, and standalone dispute contracts remain design drafts; the
active settlement contract contains its own dispute and escrow lifecycle.

```mermaid
stateDiagram-v2
  [*] --> Created
  Created --> FiatFundingConfirmed
  Created --> CryptoEscrowed
  FiatFundingConfirmed --> SettlerAssigned
  CryptoEscrowed --> SettlerAssigned
  SettlerAssigned --> SettlementInProgress
  SettlementInProgress --> SettlementConfirmed
  SettlementConfirmed --> Completed
  Created --> Cancelled
  SettlerAssigned --> Disputed
  SettlementInProgress --> Disputed
  Disputed --> Refunded
  Disputed --> Completed
```
