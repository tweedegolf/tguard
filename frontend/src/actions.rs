use serde::Deserialize;
use sha3::{Digest, Sha3_512};
use std::error::Error;
use std::fmt::{Display, Formatter};
use wasm_bindgen::prelude::JsValue;
use yew::prelude::ComponentLink;

use common::{
    AttributeValue, DownloadResult, MessageData, RecipientMessage, SealedMessage, SignResult,
};

use crate::components::receive_form::{ReceiveForm, ReceiveFormMsg};
use crate::components::send_form::{SendForm, SendFormMsg, SendFormStatus};
use crate::ibs::seal;
use crate::ibs::unseal;
use crate::js_functions::{
    download, download_bytes, get_public_key, irma_get_usk, irma_sign, send_message,
    verify_signature, IrmaSession,
};
use crate::mime::convert_from_mime;
use crate::types::{FormData, ReceivedData};

#[derive(Debug, PartialEq)]
pub enum SendError {
    MissingKey,
    FailedSeal,
    SignError,
    SerializeError,
    NotSent,
    TooLarge,
}

impl Display for SendError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SendError::MissingKey => write!(f, "public key not found"),
            SendError::SignError => write!(f, "sign failed"),
            SendError::FailedSeal => write!(f, "failed to encrypt the message"),
            SendError::SerializeError => write!(f, "failed to serialize the message"),
            SendError::NotSent => write!(f, "message could not be sent"),
            SendError::TooLarge => write!(
                f,
                "the message is too large, please limit the total size to 1 MB"
            ),
        }
    }
}

impl Error for SendError {}

pub async fn sign(link: &ComponentLink<SendForm>, message: &str) -> Result<String, SendError> {
    link.send_message(SendFormMsg::UpdateStatus(SendFormStatus::Encrypting));
    let hash = hex::encode(Sha3_512::digest(message));

    let signature = irma_sign(hash)
        .await
        .as_string()
        .ok_or(SendError::SignError)?;
    let signature: SignResult =
        serde_json::from_str(&signature).map_err(|_| SendError::SignError)?;
    let signature =
        serde_json::to_string(&signature.signature).map_err(|_| SendError::SignError)?;

    Ok(signature)
}

pub async fn encrypt_and_submit(
    link: &ComponentLink<SendForm>,
    form: FormData,
    message: String,
    signature: Option<String>,
) -> Result<(), SendError> {
    link.send_message(SendFormMsg::UpdateStatus(SendFormStatus::Encrypting));

    let public_key: String = get_public_key().await.ok_or(SendError::MissingKey)?;

    let sms: Vec<RecipientMessage> = seal(public_key, &form, message)
        .await
        .ok_or(SendError::FailedSeal)?;

    let data = MessageData {
        from: form.from.clone(),
        subject: form.subject.clone(),
        recipient_messages: sms,
        signature,
    };

    let json = serde_json::to_string(&data).map_err(|_| SendError::SerializeError)?;

    send_message(&json).await?;

    link.send_message(SendFormMsg::UpdateStatus(SendFormStatus::Sent));

    Ok(())
}

pub async fn decrypt_message(message: &SealedMessage) -> Option<String> {
    // TODO here comes multiple attribute support
    let attribute: AttributeValue = message.attributes[0].clone();

    let irma_session = IrmaSession {
        attribute_identifier: attribute.identifier.0.clone(),
        attribute_value: attribute.value.to_owned(),
        timestamp: message.timestamp,
    };

    let usk = irma_get_usk(JsValue::from_serde(&irma_session).ok()?)
        .await
        .as_string()?;

    unseal(message, usk).await
}

pub async fn download_and_decrypt(link: &ComponentLink<ReceiveForm>, id: &str) -> Option<()> {
    let message_metadata: DownloadResult = download(id).await?;
    let message_data = download_bytes(&message_metadata.content).await?;
    let message = serde_json::from_slice::<SealedMessage>(&message_data).ok()?;

    link.send_message(ReceiveFormMsg::Update(ReceivedData {
        from: message_metadata.from.clone(),
        to: message_metadata.to.clone(),
        subject: message_metadata.subject.clone(),
        message: Default::default(),
        attachments: vec![],
        attributes: message.attributes.clone(),
        signed: false,
    }));

    let pt = decrypt_message(&message).await?;

    let mut signed = false;
    // Check signature if present
    if let Some(signature) = &message_metadata.signature {
        #[derive(Deserialize)]
        struct SigData {
            message: String,
        }
        let sig_data: SigData = serde_json::from_str(signature).ok()?;
        if sig_data.message
            != format!(
                "Tguard bericht met hash {}",
                hex::encode(Sha3_512::digest(&pt))
            )
        {
            return None;
        }

        let attributes = verify_signature(signature).await?;
        if let Some(sender) = attributes.get("pbdf.sidn-pbdf.email.email") {
            if *sender != message_metadata.from {
                return None;
            }
            signed = true;
        }
    }

    // if conversion fails, use the raw message instead
    let converted = convert_from_mime(&pt).unwrap_or((pt, vec![]));

    link.send_message(ReceiveFormMsg::Update(ReceivedData {
        from: message_metadata.from.clone(),
        to: message_metadata.to,
        subject: message_metadata.subject.clone(),
        message: converted.0,
        attachments: converted.1,
        attributes: message.attributes,
        signed,
    }));

    Some(())
}
