use thiserror::Error;

use deduced_core::{
    Answer, CategoryDefinition, GuessResult, Round, RoundConfig, RoundError, RoundStatus,
};

use crate::known_facts::derive_known_facts;
use crate::result::{GameResult, game_result};
use crate::reveal::reveal_state;
use crate::state::{CategorySummary, GameStatus, GameViewState, GuessView};

#[derive(Debug, Error)]
pub enum GameError {
    #[error(transparent)]
    Round(#[from] RoundError),
}

/// Coordinates a single playable Solo round: owns the `deduced-core::Round`, and
/// turns it into renderer-ready `GameViewState`/`GameResult` so a client never has
/// to derive gameplay meaning (known facts, reveal level, ...) itself.
pub struct GameController {
    category: CategoryDefinition,
    round: Round,
}

impl GameController {
    pub fn new_solo(
        answers: &[Answer],
        category: CategoryDefinition,
        seed: u64,
    ) -> Result<Self, GameError> {
        let max_attempts = category.attempts;
        let round = Round::new(
            answers,
            RoundConfig {
                category: category.id.clone(),
                seed,
                max_attempts,
            },
        )?;

        Ok(Self { category, round })
    }

    pub fn submit_guess(&mut self, guess: &Answer) -> Result<GuessResult, GameError> {
        Ok(self.round.submit_guess(&self.category, guess)?.clone())
    }

    pub fn state(&self) -> GameViewState {
        let attempts_used = self.round.attempts_used();
        let max_attempts = self.round.max_attempts;

        GameViewState {
            category: CategorySummary {
                id: self.category.id.clone(),
                name: self.category.name.clone(),
                max_attempts,
            },
            attempts_used,
            max_attempts,
            reveal: reveal_state(attempts_used, max_attempts),
            guesses: self
                .round
                .guesses
                .iter()
                .map(|guess| GuessView {
                    answer_id: guess.answer_id.clone(),
                    answer_name: guess.answer_name.clone(),
                    comparisons: guess.comparisons.clone(),
                })
                .collect(),
            known_facts: derive_known_facts(&self.round.guesses),
            status: match self.round.status {
                RoundStatus::Playing => GameStatus::Playing,
                RoundStatus::Won => GameStatus::Won,
                RoundStatus::Lost => GameStatus::Lost,
            },
        }
    }

    pub fn result(&self) -> Option<GameResult> {
        game_result(&self.round)
    }

    pub fn round(&self) -> &Round {
        &self.round
    }

    pub fn category(&self) -> &CategoryDefinition {
        &self.category
    }
}
