use deduced_core::AttributeComparison;

use crate::known_facts::KnownFact;
use crate::reveal::RevealState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CategorySummary {
    pub id: String,
    pub name: String,
    pub max_attempts: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GuessView {
    pub answer_id: String,
    pub answer_name: String,
    pub comparisons: Vec<AttributeComparison>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameStatus {
    Playing,
    Won,
    Lost,
}

/// Everything a renderer needs to draw the current state of a Solo round, with no
/// gameplay meaning left for the client to invent.
#[derive(Debug, Clone, PartialEq)]
pub struct GameViewState {
    pub category: CategorySummary,
    pub attempts_used: usize,
    pub max_attempts: usize,
    pub reveal: RevealState,
    pub guesses: Vec<GuessView>,
    pub known_facts: Vec<KnownFact>,
    pub status: GameStatus,
}
