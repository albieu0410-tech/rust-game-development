CREATE TABLE IF NOT EXISTS daily_submissions (
    id BIGSERIAL PRIMARY KEY,
    challenge_id TEXT NOT NULL,
    player_id TEXT NOT NULL,
    won BOOLEAN NOT NULL,
    attempts_used INTEGER NOT NULL,
    max_attempts INTEGER NOT NULL,
    score INTEGER NOT NULL,
    elapsed_ms BIGINT NOT NULL,
    submitted_at BIGINT NOT NULL,
    UNIQUE (challenge_id, player_id)
);

CREATE INDEX IF NOT EXISTS idx_daily_submissions_leaderboard
    ON daily_submissions (challenge_id, won DESC, score DESC, elapsed_ms ASC);
