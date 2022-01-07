use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AttributeIdentifier(pub String);

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct DownloadResult {
    pub id: String,
    pub from: String,
    pub to: String,
    pub subject: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    pub content: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Validate)]
pub struct AttributeValue {
    pub identifier: AttributeIdentifier,
    #[validate(length(max = 256))]
    pub value: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Validate)]
pub struct SealedMessage {
    #[validate(length(min = 16, max = 32))]
    pub iv: String,
    #[validate(length(min = 16))] // max is dynamically checked server side
    pub ct: String,
    #[validate(length(min = 16, max = 1024))]
    pub c_key: String,
    pub timestamp: u64,
    #[validate]
    pub attributes: Vec<AttributeValue>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Validate)]
pub struct RecipientMessage {
    #[validate(email)]
    pub to: String,
    #[validate]
    pub sealed: SealedMessage,
}

#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
pub struct MessageData {
    #[validate(email)]
    pub from: String,
    #[validate(length(min = 1, max = 512))]
    pub subject: String,
    #[validate]
    pub recipient_messages: Vec<RecipientMessage>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[validate(length(max = 4096))]
    pub signature: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SignResult {
    pub signature: serde_json::Value,
    pub attributes: HashMap<String, String>,
}
