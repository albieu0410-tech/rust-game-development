#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Guess {
    pub answer_id: String,
}

impl Guess {
    pub fn new(answer_id: impl Into<String>) -> Self {
        Self {
            answer_id: answer_id.into(),
        }
    }
}
