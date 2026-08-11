use crate::{
    answer::Answer,
    attribute::AttributeValue,
    category::{CategoryDefinition, ComparisonRule},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Comparison {
    Match,
    Higher,
    Lower,
    Different,
    Partial,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AttributeComparison {
    pub key: String,
    pub label: String,
    pub guessed_value: AttributeValue,
    pub comparison: Comparison,
}

pub fn compare_attributes(
    category: &CategoryDefinition,
    answer: &Answer,
    guess: &Answer,
) -> Vec<AttributeComparison> {
    category
        .attributes
        .iter()
        .filter_map(|definition| {
            let answer_value = answer.attribute_value(&definition.key)?;
            let guessed_value = guess.attribute_value(&definition.key)?;
            Some(AttributeComparison {
                key: definition.key.clone(),
                label: definition.label.clone(),
                guessed_value: guessed_value.clone(),
                comparison: compare_value(definition.comparison, answer_value, guessed_value),
            })
        })
        .collect()
}

pub fn compare_value(
    rule: ComparisonRule,
    answer_value: &AttributeValue,
    guessed_value: &AttributeValue,
) -> Comparison {
    match rule {
        ComparisonRule::Exact | ComparisonRule::Bool => compare_exact(answer_value, guessed_value),
        ComparisonRule::Numeric => compare_numeric(answer_value, guessed_value),
        ComparisonRule::Tags => compare_tags(answer_value, guessed_value),
    }
}

fn compare_exact(answer_value: &AttributeValue, guessed_value: &AttributeValue) -> Comparison {
    if answer_value == guessed_value {
        Comparison::Match
    } else {
        Comparison::Different
    }
}

fn compare_numeric(answer_value: &AttributeValue, guessed_value: &AttributeValue) -> Comparison {
    let (AttributeValue::Number(answer), AttributeValue::Number(guess)) =
        (answer_value, guessed_value)
    else {
        return Comparison::Different;
    };

    if guess == answer {
        Comparison::Match
    } else if guess < answer {
        Comparison::Higher
    } else {
        Comparison::Lower
    }
}

fn compare_tags(answer_value: &AttributeValue, guessed_value: &AttributeValue) -> Comparison {
    let (AttributeValue::Tags(answer), AttributeValue::Tags(guess)) = (answer_value, guessed_value)
    else {
        return Comparison::Different;
    };

    if answer == guess {
        return Comparison::Match;
    }

    if guess.iter().any(|tag| answer.contains(tag)) {
        Comparison::Partial
    } else {
        Comparison::Different
    }
}
