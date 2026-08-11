use serde::{Deserialize, Serialize};

use crate::attribute::{Attribute, AttributeValue};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Answer {
    pub id: String,
    pub name: String,
    pub category: String,
    pub image: Option<String>,
    pub attributes: Vec<Attribute>,
}

impl Answer {
    pub fn attribute_value(&self, key: &str) -> Option<&AttributeValue> {
        self.attributes
            .iter()
            .find(|attribute| attribute.key == key)
            .map(|attribute| &attribute.value)
    }
}
