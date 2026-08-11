use crate::{Answer, CategoryDefinition};

#[derive(Debug, Clone, PartialEq)]
pub struct GameContent {
    pub categories: Vec<CategoryDefinition>,
    pub answers: Vec<Answer>,
    pub content_version: String,
}

impl GameContent {
    pub fn category(&self, id: &str) -> Option<&CategoryDefinition> {
        self.categories.iter().find(|category| category.id == id)
    }

    pub fn answers_for_category(&self, category_id: &str) -> impl Iterator<Item = &Answer> {
        self.answers
            .iter()
            .filter(move |answer| answer.category == category_id)
    }

    pub fn find_answer(&self, category_id: &str, input: &str) -> Option<&Answer> {
        let normalized = input.trim().to_lowercase();
        self.answers_for_category(category_id).find(|answer| {
            answer.id.eq_ignore_ascii_case(&normalized) || answer.name.to_lowercase() == normalized
        })
    }
}
