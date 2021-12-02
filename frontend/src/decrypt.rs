use common::SealedMessage;
use core::{convert::TryInto, fmt::Formatter};
use irmaseal_core::{Metadata, MetadataReader, MetadataReaderResult};
use js_sys::Uint8Array;
use std::fmt::Display;
use wasm_bindgen::JsValue;
use yew::services::reader::FileData;

use crate::actions::decrypt_message;
use crate::js_functions::IrmaSession;

#[derive(Debug, PartialEq)]
pub enum DecryptError {
    Deserialize,
    Failed,
    Unknown,
}

impl Display for DecryptError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DecryptError::Deserialize => write!(f, "failed to deserialize"),
            DecryptError::Failed => write!(f, "failed to decrypt"),
            DecryptError::Unknown => write!(f, "unknown file type"),
        }
    }
}

struct IrmaSealedMessage {
    metadata: Metadata,
    content: Vec<u8>,
}

pub enum MessageType {
    TguardSealedMessage,
    IrmaSealedMessage,
}

async fn parse_irmaseal_file(file: &FileData) -> Result<IrmaSealedMessage, DecryptError> {
    let mut counter = 0;
    let read_blocks_size = 32;
    let mut meta_reader = MetadataReader::new();

    let (_, header, metadata) = loop {
        match meta_reader.write(&file.content[counter..(counter + read_blocks_size)]) {
            Ok(MetadataReaderResult::Hungry) => {
                counter += read_blocks_size;
                continue;
            }
            Ok(MetadataReaderResult::Saturated {
                unconsumed,
                header,
                metadata,
            }) => break (unconsumed, header, metadata),
            Err(_e) => return Err(DecryptError::Deserialize),
        }
    };

    Ok(IrmaSealedMessage {
        metadata,
        content: (&file.content[header.len()..(file.content.len() - 32)]).to_vec(),
    })
}

async fn decrypt_irmaseal_file(sealed: &IrmaSealedMessage) -> Result<Vec<u8>, DecryptError> {
    let irma_session = IrmaSession {
        attribute_identifier: sealed.metadata.identity.attribute.atype.to_string(),
        attribute_value: sealed
            .metadata
            .identity
            .attribute
            .value
            .unwrap()
            .to_string(),
        timestamp: sealed.metadata.identity.timestamp,
    };

    let session = match JsValue::from_serde(&irma_session) {
        Ok(s) => s,
        Err(_) => return Err(DecryptError::Failed),
    };

    let usk = match crate::js_functions::irma(session).await.as_string() {
        Some(s) => s,
        None => return Err(DecryptError::Failed),
    };

    // TODO fix these unwraps?
    let usk_data: [u8; 192] = base64::decode(&usk).unwrap().try_into().unwrap();
    let usk: irmaseal_core::UserSecretKey =
        ibe::kiltz_vahlis_one::UserSecretKey::from_bytes(&usk_data)
            .unwrap()
            .into();

    let irmaseal_core::util::KeySet {
        aes_key,
        mac_key: _,
    } = sealed.metadata.derive_keys(&usk).unwrap();

    let pt =
        crate::js_functions::decrypt_cfb_hmac(&sealed.content, &aes_key, &sealed.metadata.iv).await;

    // TODO: verify SHA3-256-HMAC (file_content[(file_content.len() - 32)..]) see https://github.com/Wassasin/irmaseal/blob/master/docs/design.html
    Ok(Uint8Array::new(&pt).to_vec())
}

async fn decrypt_sealed_message(sealed: &SealedMessage) -> Result<Vec<u8>, DecryptError> {
    match decrypt_message(sealed).await {
        Some(m) => Ok(m.into_bytes()),
        None => Err(DecryptError::Failed),
    }
}

fn detect_file_type(file: &FileData) -> Option<MessageType> {
    if file.content.len() > 2 && file.content[0..2] == "{\"".to_owned().into_bytes() {
        Some(MessageType::TguardSealedMessage)
    } else if file.content.len() > 4 && file.content[0..4] == [0x14, 0x8a, 0x8e, 0xa7] {
        Some(MessageType::IrmaSealedMessage)
    } else {
        None
    }
}

pub async fn decrypt_file(file: &FileData) -> Result<Vec<u8>, DecryptError> {
    let message_type = detect_file_type(file);

    match message_type {
        Some(MessageType::TguardSealedMessage) => {
            let message = serde_json::from_slice::<SealedMessage>(&file.content).unwrap();
            decrypt_sealed_message(&message).await
        }
        Some(MessageType::IrmaSealedMessage) => {
            let message = parse_irmaseal_file(file).await.unwrap();
            decrypt_irmaseal_file(&message).await
        }
        None => Err(DecryptError::Unknown),
    }
}
