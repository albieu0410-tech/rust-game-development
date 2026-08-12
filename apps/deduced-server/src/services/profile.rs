use sqlx::PgPool;

use deduced_protocol::{ProfileSyncRequest, ProfileSyncResponse};

#[derive(sqlx::FromRow)]
struct PlayerRow {
    updated_at: i64,
    profile_json: serde_json::Value,
}

/// Last-write-wins reconciliation keyed on the client's logical clock
/// (`updated_at`). No event log or field-by-field merge — acceptable for an
/// early MVP per the docs; a real conflict-aware sync can replace this later
/// without changing the wire shape.
pub async fn sync_profile(
    pool: &PgPool,
    request: &ProfileSyncRequest,
) -> Result<ProfileSyncResponse, sqlx::Error> {
    let existing = sqlx::query_as::<_, PlayerRow>(
        "SELECT updated_at, profile_json FROM players WHERE player_id = $1",
    )
    .bind(&request.player_id)
    .fetch_optional(pool)
    .await?;

    if let Some(row) = existing
        && row.updated_at as u64 >= request.updated_at
    {
        return Ok(ProfileSyncResponse {
            updated_at: row.updated_at as u64,
            profile: row.profile_json,
            accepted: false,
        });
    }

    sqlx::query(
        "INSERT INTO players (player_id, updated_at, profile_json)
         VALUES ($1, $2, $3)
         ON CONFLICT (player_id) DO UPDATE
            SET updated_at = EXCLUDED.updated_at, profile_json = EXCLUDED.profile_json",
    )
    .bind(&request.player_id)
    .bind(request.updated_at as i64)
    .bind(&request.profile)
    .execute(pool)
    .await?;

    Ok(ProfileSyncResponse {
        updated_at: request.updated_at,
        profile: request.profile.clone(),
        accepted: true,
    })
}
