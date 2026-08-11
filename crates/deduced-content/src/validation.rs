use std::collections::HashSet;

use deduced_core::GameContent;
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
}

pub fn validate_content(content: &GameContent) -> Result<(), ContentValidationError> {
    for category in &content.categories {
        if content.answers_for_category(&category.id).next().is_none() {
            return Err(ContentValidationError::CategoryHasNoAnswers(
                category.id.clone(),
            ));
        }
    }

    for answer in &content.answers {
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
