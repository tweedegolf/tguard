use common::{AttributeIdentifier, AttributeValue};
use serde::{Deserialize, Serialize};
use yew::services::reader::FileData;

use crate::attributes::EMAIL_ATTRIBUTE_IDENTIFIER;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone)]
pub struct FormData {
    pub from: String,
    pub to: Vec<Recipient>,
    pub subject: String,
    pub message: String,
    pub attachments: Vec<FileData>,
}

impl Default for FormData {
    fn default() -> Self {
        FormData {
            from: Default::default(),
            to: vec![Default::default()],
            subject: Default::default(),
            message: Default::default(),
            attachments: vec![],
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct File {
    pub filename: String,
    pub content: Vec<u8>,
    pub mimetype: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ReceivedData {
    pub from: String,
    pub to: String,
    pub subject: String,
    pub message: String,
    pub attachments: Vec<File>,
    pub attributes: Vec<AttributeValue>,
    pub signed: bool,
}
