use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Attribute {
    pub key: String,
    pub value: AttributeValue,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AttributeValue {
    Text(String),
    Number(f64),
    Bool(bool),
    Tags(Vec<String>),
}

impl AttributeValue {
    pub fn display_value(&self) -> String {
        match self {
            Self::Text(value) => value.clone(),
            Self::Number(value) => {
                if value.fract() == 0.0 {
                    format!("{}", *value as i64)
                } else {
                    value.to_string()
                }
            }
            Self::Bool(value) => value.to_string(),
            Self::Tags(values) => values.join(", "),
        }
    }
}
