use rand::{Rng, SeedableRng, rngs::StdRng};
use thiserror::Error;

use crate::{
    answer::Answer,
    category::CategoryDefinition,
    comparison::{AttributeComparison, compare_attributes},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoundConfig {
    pub category: String,
    pub seed: u64,
    pub max_attempts: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RoundStatus {
    Playing,
    Won,
    Lost,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GuessResult {
    pub answer_id: String,
    pub answer_name: String,
    pub comparisons: Vec<AttributeComparison>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Round {
    pub answer: Answer,
    pub guesses: Vec<GuessResult>,
    pub max_attempts: usize,
    pub status: RoundStatus,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RoundError {
    #[error("no answers were provided for category {0}")]
    NoAnswersForCategory(String),
    #[error("round is already finished")]
    Finished,
    #[error("guess belongs to category {guess_category}, expected {round_category}")]
    WrongCategory {
        round_category: String,
        guess_category: String,
    },
}

impl Round {
    pub fn new(answers: &[Answer], config: RoundConfig) -> Result<Self, RoundError> {
        let category_answers = answers
            .iter()
            .filter(|answer| answer.category == config.category)
            .collect::<Vec<_>>();

        if category_answers.is_empty() {
            return Err(RoundError::NoAnswersForCategory(config.category));
        }

        let mut rng = StdRng::seed_from_u64(config.seed);
        let index = rng.random_range(0..category_answers.len());

        Ok(Self {
            answer: category_answers[index].clone(),
            guesses: Vec::new(),
            max_attempts: config.max_attempts,
            status: RoundStatus::Playing,
        })
    }

    pub fn submit_guess(
        &mut self,
        category: &CategoryDefinition,
        guess: &Answer,
    ) -> Result<&GuessResult, RoundError> {
        if self.status != RoundStatus::Playing {
            return Err(RoundError::Finished);
        }

        if guess.category != self.answer.category {
            return Err(RoundError::WrongCategory {
                round_category: self.answer.category.clone(),
                guess_category: guess.category.clone(),
            });
        }

        let comparisons = compare_attributes(category, &self.answer, guess);
        let result = GuessResult {
            answer_id: guess.id.clone(),
            answer_name: guess.name.clone(),
            comparisons,
        };

        self.guesses.push(result);

        if guess.id == self.answer.id {
            self.status = RoundStatus::Won;
        } else if self.guesses.len() >= self.max_attempts {
            self.status = RoundStatus::Lost;
        }

        Ok(self.guesses.last().expect("guess was just pushed"))
    }

    pub fn attempts_used(&self) -> usize {
        self.guesses.len()
    }

    pub fn reveal_level(&self) -> usize {
        self.guesses.len().min(self.max_attempts.saturating_sub(1))
    }
}
