use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CategoryDefinition {
    pub id: String,
    pub name: String,
    pub attempts: usize,
    pub attributes: Vec<AttributeDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AttributeDefinition {
    pub key: String,
    pub label: String,
    pub comparison: ComparisonRule,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ComparisonRule {
    Exact,
    Numeric,
    Tags,
    Bool,
}
