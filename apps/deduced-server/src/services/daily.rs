use std::time::{SystemTime, UNIX_EPOCH};

use sqlx::PgPool;
use thiserror::Error;

use deduced_core::{
    CategoryDefinition, GameContent, Round, RoundConfig, RoundError, RoundStatus, score_round,
};
use deduced_protocol::{DailySubmissionResult, LeaderboardEntry};

/// Days since the Unix epoch, used to deterministically rotate the daily
/// category and seed. Every server instance and every client computes the
/// same value for "today" without needing to agree on a shared clock beyond
/// wall time.
pub fn today_day_index() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock is before the Unix epoch")
        .as_secs()
        / 86_400
}

pub fn category_for_day(day_index: u64, categories: &[CategoryDefinition]) -> &CategoryDefinition {
    let index = (day_index as usize) % categories.len();
    &categories[index]
}

/// Deterministic (not random) seed derived from the day index, so every
/// player reconstructs the exact same round for "today".
pub fn seed_for_day(day_index: u64) -> u64 {
    fnv1a(&day_index.to_le_bytes())
}

fn fnv1a(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for &byte in bytes {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

pub fn challenge_id_for(day_index: u64, category_id: &str) -> String {
    format!("daily-{day_index}-{category_id}")
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ChallengeIdError {
    #[error("malformed challenge id: {0}")]
    Malformed(String),
}

pub fn parse_challenge_id(challenge_id: &str) -> Result<(u64, String), ChallengeIdError> {
    let rest = challenge_id
        .strip_prefix("daily-")
        .ok_or_else(|| ChallengeIdError::Malformed(challenge_id.to_string()))?;
    let (day_index_str, category_id) = rest
        .split_once('-')
        .ok_or_else(|| ChallengeIdError::Malformed(challenge_id.to_string()))?;
    let day_index = day_index_str
        .parse::<u64>()
        .map_err(|_| ChallengeIdError::Malformed(challenge_id.to_string()))?;

    Ok((day_index, category_id.to_string()))
}

#[derive(Debug, Error)]
pub enum ReplayError {
    #[error("unknown category {0}")]
    UnknownCategory(String),
    #[error("no answer matches id {0}")]
    UnknownAnswer(String),
    #[error(transparent)]
    Round(#[from] RoundError),
}

/// Replays a submitted guess sequence through the shared rules engine and
/// returns the authoritative outcome. The client's own claimed result is
/// never trusted directly.
pub fn replay_submission(
    content: &GameContent,
    category_id: &str,
    seed: u64,
    guesses: &[String],
) -> Result<DailySubmissionResult, ReplayError> {
    let category = content
        .category(category_id)
        .ok_or_else(|| ReplayError::UnknownCategory(category_id.to_string()))?;

    let mut round = Round::new(
        &content.answers,
        RoundConfig {
            category: category.id.clone(),
            seed,
            max_attempts: category.attempts,
        },
    )?;

    for answer_id in guesses {
        if round.status != RoundStatus::Playing {
            break;
        }

        let answer = content
            .answers_for_category(category_id)
            .find(|answer| &answer.id == answer_id)
            .ok_or_else(|| ReplayError::UnknownAnswer(answer_id.clone()))?;

        round.submit_guess(category, answer)?;
    }

    let score = score_round(&round);

    Ok(DailySubmissionResult {
        won: round.status == RoundStatus::Won,
        attempts_used: round.attempts_used(),
        max_attempts: round.max_attempts,
        score: score.points,
    })
}

#[derive(sqlx::FromRow)]
struct SubmissionRow {
    player_id: String,
    won: bool,
    score: i32,
    attempts_used: i32,
    elapsed_ms: i64,
}

/// Stores a validated submission. Returns `Ok(true)` if newly recorded, or
/// `Ok(false)` if this player already has a submission for this challenge
/// (each player gets one Daily attempt).
pub async fn record_submission(
    pool: &PgPool,
    challenge_id: &str,
    player_id: &str,
    result: &DailySubmissionResult,
    elapsed_ms: u64,
) -> Result<bool, sqlx::Error> {
    let submitted_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock is before the Unix epoch")
        .as_secs() as i64;

    let outcome = sqlx::query(
        "INSERT INTO daily_submissions
            (challenge_id, player_id, won, attempts_used, max_attempts, score, elapsed_ms, submitted_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
         ON CONFLICT (challenge_id, player_id) DO NOTHING",
    )
    .bind(challenge_id)
    .bind(player_id)
    .bind(result.won)
    .bind(result.attempts_used as i32)
    .bind(result.max_attempts as i32)
    .bind(result.score as i32)
    .bind(elapsed_ms as i64)
    .bind(submitted_at)
    .execute(pool)
    .await?;

    Ok(outcome.rows_affected() == 1)
}

pub async fn leaderboard(
    pool: &PgPool,
    challenge_id: &str,
    limit: i64,
) -> Result<Vec<LeaderboardEntry>, sqlx::Error> {
    let rows = sqlx::query_as::<_, SubmissionRow>(
        "SELECT player_id, won, score, attempts_used, elapsed_ms
         FROM daily_submissions
         WHERE challenge_id = $1
         ORDER BY won DESC, score DESC, elapsed_ms ASC
         LIMIT $2",
    )
    .bind(challenge_id)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| LeaderboardEntry {
            player_id: row.player_id,
            won: row.won,
            score: row.score as u32,
            attempts_used: row.attempts_used as usize,
            elapsed_ms: row.elapsed_ms as u64,
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn challenge_id_round_trips_through_parsing() {
        let id = challenge_id_for(19_876, "countries");
        assert_eq!(id, "daily-19876-countries");
        assert_eq!(
            parse_challenge_id(&id),
            Ok((19_876, "countries".to_string()))
        );
    }

    #[test]
    fn parse_challenge_id_rejects_malformed_input() {
        assert!(parse_challenge_id("not-a-daily-id").is_err());
        assert!(parse_challenge_id("daily-notanumber-cars").is_err());
    }

    #[test]
    fn seed_for_day_is_deterministic_and_varies_by_day() {
        assert_eq!(seed_for_day(100), seed_for_day(100));
        assert_ne!(seed_for_day(100), seed_for_day(101));
    }

    #[test]
    fn category_for_day_rotates_through_the_list() {
        let categories = vec![
            test_category("cars"),
            test_category("companies"),
            test_category("countries"),
        ];

        assert_eq!(category_for_day(0, &categories).id, "cars");
        assert_eq!(category_for_day(1, &categories).id, "companies");
        assert_eq!(category_for_day(2, &categories).id, "countries");
        assert_eq!(category_for_day(3, &categories).id, "cars");
    }

    fn test_category(id: &str) -> CategoryDefinition {
        CategoryDefinition {
            id: id.to_string(),
            name: id.to_string(),
            attempts: 5,
            attributes: vec![],
        }
    }

    fn fixture_content() -> GameContent {
        use deduced_core::{
            Answer, Attribute, AttributeDefinition, AttributeValue, ComparisonRule,
        };

        let category = CategoryDefinition {
            id: "cars".to_string(),
            name: "Cars".to_string(),
            attempts: 3,
            attributes: vec![AttributeDefinition {
                key: "country".to_string(),
                label: "Country".to_string(),
                comparison: ComparisonRule::Exact,
            }],
        };

        let answer = |id: &str, name: &str, country: &str| Answer {
            id: id.to_string(),
            name: name.to_string(),
            category: "cars".to_string(),
            image: None,
            attributes: vec![Attribute {
                key: "country".to_string(),
                value: AttributeValue::Text(country.to_string()),
            }],
        };

        GameContent {
            categories: vec![category],
            answers: vec![
                answer("car_honda", "Honda", "Japan"),
                answer("car_volvo", "Volvo", "Sweden"),
                answer("car_bmw", "BMW", "Germany"),
            ],
            content_version: "test".to_string(),
        }
    }

    #[test]
    fn replay_submission_ignores_client_claimed_result_and_computes_its_own() {
        let content = fixture_content();
        let seed = 12_345;

        // Find the true answer the way the server would, then replay guessing it directly.
        let target_id = Round::new(
            &content.answers,
            RoundConfig {
                category: "cars".to_string(),
                seed,
                max_attempts: 3,
            },
        )
        .unwrap()
        .answer
        .id;

        let result = replay_submission(&content, "cars", seed, std::slice::from_ref(&target_id))
            .expect("replay ok");

        assert!(result.won);
        assert_eq!(result.attempts_used, 1);
    }

    #[test]
    fn replay_submission_rejects_unknown_answer_ids() {
        let content = fixture_content();
        let result = replay_submission(&content, "cars", 1, &["not_a_real_id".to_string()]);
        assert!(matches!(result, Err(ReplayError::UnknownAnswer(_))));
    }

    #[test]
    fn replay_submission_stops_early_once_the_round_is_decided() {
        let content = fixture_content();
        let seed = 12_345;

        let target_id = Round::new(
            &content.answers,
            RoundConfig {
                category: "cars".to_string(),
                seed,
                max_attempts: 3,
            },
        )
        .unwrap()
        .answer
        .id;

        // Guesses after the winning one should be ignored, not error out or overwrite the result.
        let all_ids: Vec<String> = content.answers.iter().map(|a| a.id.clone()).collect();
        let mut guesses = vec![target_id];
        guesses.extend(all_ids);

        let result = replay_submission(&content, "cars", seed, &guesses).expect("replay ok");
        assert!(result.won);
        assert_eq!(result.attempts_used, 1);
    }
}
