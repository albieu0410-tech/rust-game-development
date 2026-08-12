use deduced_core::{Round, RoundStatus, Score, score_round};

/// The finished outcome of a round: who won, what the answer was, and the score.
#[derive(Debug, Clone, PartialEq)]
pub struct GameResult {
    pub won: bool,
    pub category_id: String,
    pub answer_id: String,
    pub answer_name: String,
    pub attempts_used: usize,
    pub max_attempts: usize,
    pub score: Score,
}

/// `None` while the round is still in progress; `Some` once it has been won or lost.
pub fn game_result(round: &Round) -> Option<GameResult> {
    if round.status == RoundStatus::Playing {
        return None;
    }

    Some(GameResult {
        won: round.status == RoundStatus::Won,
        category_id: round.answer.category.clone(),
        answer_id: round.answer.id.clone(),
        answer_name: round.answer.name.clone(),
        attempts_used: round.attempts_used(),
        max_attempts: round.max_attempts,
        score: score_round(round),
    })
}
