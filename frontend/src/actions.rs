use std::error::Error;
use std::fmt::{Display, Formatter};
use wasm_bindgen::prelude::JsValue;
use yew::prelude::ComponentLink;

use common::{AttributeValue, DownloadResult, MessageData, RecipientMessage, SealedMessage};

use crate::components::receive_form::{ReceiveForm, ReceiveFormMsg};
use crate::components::send_form::{SendForm, SendFormMsg, SendFormStatus};
use crate::ibs::seal;
use crate::ibs::unseal;
use crate::js_functions::{
    download, download_bytes, get_public_key, irma, send_message, IrmaSession,
};
use crate::mime::convert_from_mime;
use crate::types::{FormData, ReceivedData};

#[derive(Debug, PartialEq)]
pub enum SendError {
    MissingKey,
    FailedSeal,
    SerializeError,
    NotSent,
}

impl Display for SendError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SendError::MissingKey => write!(f, "public key not found"),
            SendError::FailedSeal => write!(f, "failed to seal message"),
            SendError::SerializeError => write!(f, "failed to serialize"),
            SendError::NotSent => write!(f, "message could not be sent"),
        }
    }
}

impl Error for SendError {}

pub async fn encrypt_and_submit(
    link: &ComponentLink<SendForm>,
    form: FormData,
) -> Result<(), SendError> {
    link.send_message(SendFormMsg::UpdateStatus(SendFormStatus::Encrypting));

    let public_key: String = get_public_key().await.ok_or(SendError::MissingKey)?;

    let sms: Vec<RecipientMessage> = seal(public_key, &form).await.ok_or(SendError::FailedSeal)?;

    let data = MessageData {
        from: form.from.clone(),
        subject: form.subject.clone(),
        recipient_messages: sms,
    };

    let json = serde_json::to_string(&data).map_err(|_| SendError::SerializeError)?;

    if send_message(&json).await.is_none() {
        return Err(SendError::NotSent);
    }

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

    let usk = irma(JsValue::from_serde(&irma_session).ok()?)
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
    }));

    let pt = decrypt_message(&message).await?;

    // if conversion fails, use the raw message instead
    let converted = convert_from_mime(&pt).unwrap_or((pt, vec![]));

    link.send_message(ReceiveFormMsg::Update(ReceivedData {
        from: message_metadata.from.clone(),
        to: message_metadata.to,
        subject: message_metadata.subject.clone(),
        message: converted.0,
        attachments: converted.1,
        attributes: message.attributes,
    }));

    Some(())
}
