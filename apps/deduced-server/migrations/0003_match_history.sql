CREATE TABLE IF NOT EXISTS match_history (
    id BIGSERIAL PRIMARY KEY,
    match_id TEXT NOT NULL UNIQUE,
    category_id TEXT NOT NULL,
    player_a TEXT NOT NULL,
    player_b TEXT NOT NULL,
    winner_id TEXT,
    finished_at BIGINT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_match_history_player_a ON match_history (player_a);
CREATE INDEX IF NOT EXISTS idx_match_history_player_b ON match_history (player_b);
