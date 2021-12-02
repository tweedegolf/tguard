use common::{AttributeIdentifier, AttributeValue};
use serde::{Deserialize, Serialize};

use crate::attributes::EMAIL_ATTRIBUTE_IDENTIFIER;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Recipient {
    pub to: String,
    pub attributes: Vec<AttributeValue>,
}

impl Default for Recipient {
    fn default() -> Self {
        Recipient {
            to: Default::default(),
            attributes: vec![
                AttributeValue {
                    identifier: AttributeIdentifier(EMAIL_ATTRIBUTE_IDENTIFIER.to_owned()),
                    value: String::new(),
                };
                1
            ],
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FormData {
    pub from: String,
    pub to: Vec<Recipient>,
    pub subject: String,
    pub message: String,
}

impl Default for FormData {
    fn default() -> Self {
        FormData {
            from: Default::default(),
            to: vec![Default::default()],
            subject: Default::default(),
            message: Default::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ReceivedData {
    pub from: String,
    pub to: String,
    pub subject: String,
    pub message: String,
    pub attributes: Vec<AttributeValue>,
}
