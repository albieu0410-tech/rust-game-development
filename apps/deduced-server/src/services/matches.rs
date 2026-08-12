use std::time::{SystemTime, UNIX_EPOCH};

use sqlx::PgPool;

use deduced_protocol::MatchHistoryEntry;

pub async fn record_match_result(
    pool: &PgPool,
    match_id: &str,
    category_id: &str,
    player_a: &str,
    player_b: &str,
    winner_id: Option<&str>,
) -> Result<(), sqlx::Error> {
    let finished_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock is before the Unix epoch")
        .as_secs() as i64;

    sqlx::query(
        "INSERT INTO match_history
            (match_id, category_id, player_a, player_b, winner_id, finished_at)
         VALUES ($1, $2, $3, $4, $5, $6)
         ON CONFLICT (match_id) DO NOTHING",
    )
    .bind(match_id)
    .bind(category_id)
    .bind(player_a)
    .bind(player_b)
    .bind(winner_id)
    .bind(finished_at)
    .execute(pool)
    .await?;

    Ok(())
}

#[derive(sqlx::FromRow)]
struct MatchHistoryRow {
    match_id: String,
    category_id: String,
    player_a: String,
    player_b: String,
    winner_id: Option<String>,
    finished_at: i64,
}

pub async fn history_for_player(
    pool: &PgPool,
    player_id: &str,
    limit: i64,
) -> Result<Vec<MatchHistoryEntry>, sqlx::Error> {
    let rows = sqlx::query_as::<_, MatchHistoryRow>(
        "SELECT match_id, category_id, player_a, player_b, winner_id, finished_at
         FROM match_history
         WHERE player_a = $1 OR player_b = $1
         ORDER BY finished_at DESC
         LIMIT $2",
    )
    .bind(player_id)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| MatchHistoryEntry {
            match_id: row.match_id,
            category_id: row.category_id,
            player_a: row.player_a,
            player_b: row.player_b,
            winner_id: row.winner_id,
            finished_at: row.finished_at as u64,
        })
        .collect())
}
