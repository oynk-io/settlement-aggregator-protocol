CREATE TABLE payment_events (
  id BIGSERIAL PRIMARY KEY,
  ledger BIGINT NOT NULL,
  tx_hash TEXT NOT NULL,
  contract_id TEXT NOT NULL,
  payment_id BIGINT,
  event_type TEXT NOT NULL,
  actor TEXT,
  amount NUMERIC,
  asset_code TEXT,
  payload JSONB NOT NULL,
  created_at TIMESTAMPTZ DEFAULT now()
);

CREATE TABLE settler_metrics (
  settler TEXT PRIMARY KEY,
  completed BIGINT DEFAULT 0,
  failed BIGINT DEFAULT 0,
  total_volume_usd NUMERIC DEFAULT 0,
  reputation INT DEFAULT 500,
  updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX idx_payment_events_payment_id ON payment_events(payment_id);
CREATE INDEX idx_payment_events_type ON payment_events(event_type);
CREATE INDEX idx_payment_events_ledger ON payment_events(ledger);
