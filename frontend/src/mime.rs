use lettre::message::{header::ContentType, Attachment, MultiPart, SinglePart};
use mail_parser::{BodyPart, Message, MessagePart, MimeHeaders};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use std::iter::repeat_with;

use crate::types::{File, FormData};

// Based on: https://docs.rs/lettre/0.10.0-rc.4/src/lettre/message/mimebody.rs.html#174 , but without fastrand
fn make_boundary() -> String {
    let mut rng = thread_rng();
    repeat_with(|| rng.sample(Alphanumeric)).take(40).collect()
}

pub fn convert_to_mime(form: &FormData) -> String {
    let text = SinglePart::plain(form.message.clone());

    // manually set the ContenType and boundary because fastrand gives problems
    let boundary = make_boundary();
    let mut content = MultiPart::builder()
        .header(
            ContentType::parse(format!("multipart/mixed; boundary={}", boundary).as_ref()).unwrap(),
        )
        .singlepart(text);

    for attachment in &form.attachments {
        // try to guess the mime type from the file name
        let mime_type = mime_guess::from_path(&attachment.name)
            .first_raw()
            .unwrap_or("application/octet-stream");
        let part = Attachment::new(attachment.name.clone()).body(
            attachment.content.clone(),
            ContentType::parse(mime_type).unwrap(),
        );
        content = content.singlepart(part.clone());
    }

    String::from_utf8(content.formatted()).unwrap()
}

// functionality to parse a raw mime email message
fn get_filename<'a, T>(part: &T) -> Option<String>
where
    T: MimeHeaders<'a>,
{
    Some(
        part.get_content_disposition()?
            .get_attribute("filename")?
            .to_string(),
    )
}

fn get_content_type<'a, T>(part: &T) -> Option<String>
where
    T: MimeHeaders<'a>,
{
    let content_type = part.get_content_type()?;
    match content_type.get_subtype() {
        Some(s) => Some(format!("{}/{}", content_type.get_type(), s)),
        None => Some(content_type.get_type().to_string()),
    }
}

fn parse_attachment(attachment: &MessagePart) -> Option<File> {
    match attachment {
        MessagePart::Text(m) => Some(File {
            filename: get_filename(m).unwrap_or_else(|| "attachment.txt".to_owned()),
            content: m.get_contents().to_vec(),
            mimetype: get_content_type(m).unwrap_or_else(|| "text/plain".to_owned()),
        }),
        MessagePart::Binary(m) => Some(File {
            filename: get_filename(m).unwrap_or_else(|| "attachment.bin".to_owned()),
            content: m.get_contents().to_vec(),
            mimetype: get_content_type(m).unwrap_or_else(|| "application/octet-stream".to_owned()),
        }),
        // Nested RFC5322/RFC822 message, add it as an attachment
        MessagePart::Message(m) => Some(File {
            filename: get_filename(m).unwrap_or_else(|| "attachment.eml".to_owned()),
            content: m.get_contents().to_vec(),
            mimetype: "message/rfc822".to_owned(),
        }),
        _ => None,
    }
}

pub fn convert_from_mime(message: &str) -> Option<(String, Vec<File>)> {
    let email = Message::parse(message.as_bytes())?;

    let mut attachments: Vec<File> = vec![];
    for attachment in email.get_attachments() {
        if let Some(a) = parse_attachment(attachment) {
            attachments.push(a)
        }
    }

    let body = email.get_text_body(0)?.to_string();
    Some((body, attachments))
}
