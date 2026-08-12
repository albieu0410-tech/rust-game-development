use std::collections::{HashMap, HashSet};

use deduced_core::{AttributeValue, GameContent};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ContentValidationError {
    #[error("category {0} has no answers")]
    CategoryHasNoAnswers(String),
    #[error("answer {answer_id} references unknown category {category_id}")]
    UnknownCategory {
        answer_id: String,
        category_id: String,
    },
    #[error("answer {answer_id} has undeclared attribute {attribute_key}")]
    UndeclaredAttribute {
        answer_id: String,
        attribute_key: String,
    },
    #[error("answer {answer_id} is missing attribute {attribute_key}")]
    MissingAttribute {
        answer_id: String,
        attribute_key: String,
    },
    #[error("duplicate category id {0}")]
    DuplicateCategoryId(String),
    #[error("duplicate answer id {0}")]
    DuplicateAnswerId(String),
    #[error("category {category_id} has two answers both named {name}")]
    DuplicateAnswerName { category_id: String, name: String },
    #[error("category {0} has an empty name")]
    EmptyCategoryName(String),
    #[error("answer {0} has an empty name")]
    EmptyAnswerName(String),
    #[error("answer {answer_id} has an empty value for attribute {attribute_key}")]
    EmptyAttributeValue {
        answer_id: String,
        attribute_key: String,
    },
}

pub fn validate_content(content: &GameContent) -> Result<(), ContentValidationError> {
    let mut seen_category_ids = HashSet::new();
    for category in &content.categories {
        if !seen_category_ids.insert(category.id.as_str()) {
            return Err(ContentValidationError::DuplicateCategoryId(
                category.id.clone(),
            ));
        }
        if category.name.trim().is_empty() {
            return Err(ContentValidationError::EmptyCategoryName(
                category.id.clone(),
            ));
        }
        if content.answers_for_category(&category.id).next().is_none() {
            return Err(ContentValidationError::CategoryHasNoAnswers(
                category.id.clone(),
            ));
        }
    }

    let mut seen_answer_ids = HashSet::new();
    let mut seen_names_per_category: HashMap<&str, HashSet<&str>> = HashMap::new();

    for answer in &content.answers {
        if !seen_answer_ids.insert(answer.id.as_str()) {
            return Err(ContentValidationError::DuplicateAnswerId(answer.id.clone()));
        }
        if answer.name.trim().is_empty() {
            return Err(ContentValidationError::EmptyAnswerName(answer.id.clone()));
        }
        if !seen_names_per_category
            .entry(answer.category.as_str())
            .or_default()
            .insert(answer.name.as_str())
        {
            return Err(ContentValidationError::DuplicateAnswerName {
                category_id: answer.category.clone(),
                name: answer.name.clone(),
            });
        }

        let Some(category) = content.category(&answer.category) else {
            return Err(ContentValidationError::UnknownCategory {
                answer_id: answer.id.clone(),
                category_id: answer.category.clone(),
            });
        };

        let declared = category
            .attributes
            .iter()
            .map(|attribute| attribute.key.as_str())
            .collect::<HashSet<_>>();
        let present = answer
            .attributes
            .iter()
            .map(|attribute| attribute.key.as_str())
            .collect::<HashSet<_>>();

        for attribute in &answer.attributes {
            if !declared.contains(attribute.key.as_str()) {
                return Err(ContentValidationError::UndeclaredAttribute {
                    answer_id: answer.id.clone(),
                    attribute_key: attribute.key.clone(),
                });
            }
            if is_empty_value(&attribute.value) {
                return Err(ContentValidationError::EmptyAttributeValue {
                    answer_id: answer.id.clone(),
                    attribute_key: attribute.key.clone(),
                });
            }
        }

        for attribute in &category.attributes {
            if !present.contains(attribute.key.as_str()) {
                return Err(ContentValidationError::MissingAttribute {
                    answer_id: answer.id.clone(),
                    attribute_key: attribute.key.clone(),
                });
            }
        }
    }

    Ok(())
}

fn is_empty_value(value: &AttributeValue) -> bool {
    match value {
        AttributeValue::Text(text) => text.trim().is_empty(),
        AttributeValue::Tags(tags) => {
            tags.is_empty() || tags.iter().any(|tag| tag.trim().is_empty())
        }
        AttributeValue::Number(_) | AttributeValue::Bool(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use deduced_core::{
        Answer, Attribute, AttributeDefinition, CategoryDefinition, ComparisonRule,
    };

    fn category() -> CategoryDefinition {
        CategoryDefinition {
            id: "cars".to_string(),
            name: "Cars".to_string(),
            attempts: 5,
            attributes: vec![AttributeDefinition {
                key: "country".to_string(),
                label: "Country".to_string(),
                comparison: ComparisonRule::Exact,
            }],
        }
    }

    fn answer(id: &str, name: &str, country: &str) -> Answer {
        Answer {
            id: id.to_string(),
            name: name.to_string(),
            category: "cars".to_string(),
            image: None,
            attributes: vec![Attribute {
                key: "country".to_string(),
                value: AttributeValue::Text(country.to_string()),
            }],
        }
    }

    fn content(answers: Vec<Answer>) -> GameContent {
        GameContent {
            categories: vec![category()],
            answers,
            content_version: "test".to_string(),
        }
    }

    #[test]
    fn valid_content_passes() {
        let result = validate_content(&content(vec![answer("car_a", "A", "Japan")]));
        assert!(result.is_ok());
    }

    #[test]
    fn rejects_duplicate_answer_ids() {
        let result = validate_content(&content(vec![
            answer("car_a", "A", "Japan"),
            answer("car_a", "B", "Germany"),
        ]));
        assert_eq!(
            result,
            Err(ContentValidationError::DuplicateAnswerId(
                "car_a".to_string()
            ))
        );
    }

    #[test]
    fn rejects_duplicate_answer_names_within_a_category() {
        let result = validate_content(&content(vec![
            answer("car_a", "Same Name", "Japan"),
            answer("car_b", "Same Name", "Germany"),
        ]));
        assert_eq!(
            result,
            Err(ContentValidationError::DuplicateAnswerName {
                category_id: "cars".to_string(),
                name: "Same Name".to_string(),
            })
        );
    }

    #[test]
    fn rejects_empty_answer_name() {
        let result = validate_content(&content(vec![answer("car_a", "  ", "Japan")]));
        assert_eq!(
            result,
            Err(ContentValidationError::EmptyAnswerName("car_a".to_string()))
        );
    }

    #[test]
    fn rejects_empty_attribute_value() {
        let result = validate_content(&content(vec![answer("car_a", "A", "  ")]));
        assert_eq!(
            result,
            Err(ContentValidationError::EmptyAttributeValue {
                answer_id: "car_a".to_string(),
                attribute_key: "country".to_string(),
            })
        );
    }

    #[test]
    fn rejects_duplicate_category_ids() {
        let mut game_content = content(vec![answer("car_a", "A", "Japan")]);
        game_content.categories.push(category());

        let result = validate_content(&game_content);
        assert_eq!(
            result,
            Err(ContentValidationError::DuplicateCategoryId(
                "cars".to_string()
            ))
        );
    }
}
