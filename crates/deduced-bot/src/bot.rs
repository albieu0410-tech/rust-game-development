use deduced_core::{Answer, Round};

use crate::BotDifficulty;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bot {
    pub difficulty: BotDifficulty,
}

impl Bot {
    pub fn choose_guess<'a>(&self, round: &Round, answers: &'a [Answer]) -> Option<&'a Answer> {
        answers
            .iter()
            .filter(|answer| answer.category == round.answer.category)
            .find(|answer| {
                !round
                    .guesses
                    .iter()
                    .any(|guess| guess.answer_id == answer.id)
            })
    }
}
