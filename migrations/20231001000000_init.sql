CREATE TABLE urls (
    id VARCHAR(20) PRIMARY KEY,
    original_url TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expiration_time TIMESTAMPTZ,
    click_count INTEGER,
    destruction_mode JSONB NOT NULL
);

CREATE INDEX idx_destruction ON urls USING GIN (destruction_mode);