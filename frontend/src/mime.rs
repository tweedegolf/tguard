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
        let part = Attachment::new(attachment.name.clone()).body(
            attachment.content.clone(),
            ContentType::parse("application/octet-stream").unwrap(),
        );
        content = content.singlepart(part.clone());
    }

    String::from_utf8(content.formatted()).unwrap()
}

pub fn convert_from_mime(message: &str) -> Result<(String, Vec<File>), ()> {
    let email = Message::parse(message.as_bytes()).unwrap();

    let mut attachments: Vec<File> = vec![];
    for attachment in email.get_attachments() {
        match attachment {
            // add attachments of type Text as .txt file
            MessagePart::Text(m) => attachments.push(File {
                filename: "attachment.txt".to_string(),
                content: m.get_contents().to_vec(),
            }),
            MessagePart::Binary(m) => {
                if let Some(name) = m
                    .get_content_disposition()
                    .unwrap()
                    .get_attribute("filename")
                {
                    attachments.push(File {
                        filename: name.to_string(),
                        content: m.get_contents().to_vec(),
                    })
                } else {
                    attachments.push(File {
                        filename: "attachment.txt".to_string(),
                        content: m.get_contents().to_vec(),
                    })
                }
            }
            MessagePart::InlineBinary(m) => {
                if let Some(name) = m
                    .get_content_disposition()
                    .unwrap()
                    .get_attribute("filename")
                {
                    attachments.push(File {
                        filename: name.to_string(),
                        content: m.get_contents().to_vec(),
                    })
                } else {
                    attachments.push(File {
                        filename: "attachment.txt".to_string(),
                        content: m.get_contents().to_vec(),
                    })
                }
            }
            MessagePart::Message(m) => {
                // this is a nested mime message, add it as an attachment
                attachments.push(File {
                    filename: "nested.txt".to_string(),
                    content: m
                        .get_text_body(0)
                        .unwrap()
                        .to_string()
                        .as_bytes()
                        .to_owned(),
                })
            }
        };
    }

    let body = email.get_text_body(0).unwrap().to_string();
    Ok((body, attachments))
}
