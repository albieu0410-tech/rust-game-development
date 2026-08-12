CREATE TABLE IF NOT EXISTS players (
    player_id TEXT PRIMARY KEY,
    updated_at BIGINT NOT NULL,
    profile_json JSONB NOT NULL
);
