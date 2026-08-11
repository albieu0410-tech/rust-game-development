use crate::{Round, RoundStatus};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Score {
    pub points: u32,
}

pub fn score_round(round: &Round) -> Score {
    let points = match round.status {
        RoundStatus::Won => {
            let remaining = round.max_attempts.saturating_sub(round.attempts_used());
            100 + (remaining as u32 * 25)
        }
        RoundStatus::Playing | RoundStatus::Lost => 0,
    };

    Score { points }
}
