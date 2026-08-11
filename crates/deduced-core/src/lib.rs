pub mod answer;
pub mod attribute;
pub mod category;
pub mod comparison;
pub mod game;
pub mod guess;
pub mod round;
pub mod scoring;

pub use answer::Answer;
pub use attribute::{Attribute, AttributeValue};
pub use category::{AttributeDefinition, CategoryDefinition, ComparisonRule};
pub use comparison::{AttributeComparison, Comparison, compare_attributes, compare_value};
pub use game::GameContent;
pub use guess::Guess;
pub use round::{GuessResult, Round, RoundConfig, RoundError, RoundStatus};
pub use scoring::{Score, score_round};
