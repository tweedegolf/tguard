use rocket::form::Form;
use rocket::State;
use serde::Deserialize;

use crate::{config::Config, email::send_confirmation_email, error::Error, id::Id, Database};

#[derive(Deserialize, Debug)]
struct MailgunAttachment {
    #[serde(alias = "content-type")]
    content_type: String,
    name: String,
    url: String,
    size: u64,
}

#[derive(Deserialize, Debug)]
struct MailgunMessage {
    sender: String,
    subject: String,
    #[serde(alias = "body-plain")]
    body_plain: String,
    #[serde(alias = "Message-Id")]
    message_id: String,
    attachments: Vec<MailgunAttachment>,
}

#[derive(Deserialize, FromForm, Debug)]
pub struct MailgunNotification {
    #[serde(alias = "message-url")]
    #[field(name = "message-url")]
    message_url: String,
}

#[derive(Deserialize, Debug)]
struct MailgunStorage {
    url: String,
}

#[derive(Deserialize, Debug)]
struct MailgunEvent {
    storage: MailgunStorage,
}

#[derive(Deserialize, Debug)]
struct MailgunEvents {
    items: Vec<MailgunEvent>,
}

fn extract_original_from(message_text: &str) -> Option<&str> {
    // Get the original from line
    let from_line_start = message_text.find("\r\nFrom: ")? + 8;
    let from_line_end = message_text[from_line_start..].find("\r\n")? + from_line_start;
    let from_line = message_text[from_line_start..from_line_end].trim();

    // Extract the email adress from that line in case `Name <email@example.com>` notation is used
    if let Some(email_start) = from_line.find('<') {
        let email_end = from_line[email_start..].find('>')? + email_start;
        Some(&from_line[email_start + 1..email_end])
    } else {
        Some(from_line)
    }
}

async fn process_message(config: &Config, conn: &Database, url: &str) -> Result<(), Error> {
    // verify message url format
    if !url.starts_with(&config.mailgun_message_url_prefix) {
        return Err(Error::InvalidApiUrl);
    }

    let client = reqwest::Client::new();

    let message: MailgunMessage = client
        .get(url)
        .basic_auth("api", config.mailgun_key.as_ref())
        .send()
        .await?
        .json()
        .await?;

    let attachment = (|| {
        for attachment in &message.attachments {
            if attachment.content_type == "application/irmaseal"
                || attachment.name == "encrypted.irmaseal"
            {
                return Some(attachment);
            }
        }
        None
    })()
    .ok_or(Error::MissingData)?;

    let id = Id::new();
    let to = &message.sender;
    let subject = &message.subject;
    let from = extract_original_from(&message.body_plain).unwrap_or(&config.from_fallback);

    let row_id = id.to_string();
    let to_copy = to.to_string();
    let from_copy = from.to_string();
    let subject_copy = subject.clone();

    conn.run(move |c| -> Result<(), Error> {
        c.execute(
            "INSERT INTO messages (id, from_address, to_address, subject) VALUES ($1, $2, $3, $4)",
            &[&row_id, &from_copy, &to_copy, &subject_copy],
        )?;
        Ok(())
    })
    .await?;

    let attachment = client
        .get(&attachment.url)
        .basic_auth("api", config.mailgun_key.as_ref())
        .send()
        .await?
        .bytes()
        .await?;

    config
        .storage
        .store(attachment.to_vec(), &id.to_string())
        .await?;

    send_confirmation_email(config, id, to, subject)?;

    Ok(())
}

// Process a store notification as sent by mailgun
#[post("/api/newemail", data = "<notification>")]
pub async fn new_email(
    config: &State<Config>,
    conn: Database,
    notification: Form<MailgunNotification>,
) -> Result<(), Error> {
    process_message(config, &conn, &notification.message_url).await
}

// This is probably not the best long term solution, but for testing now is still reasonable
// Long term we might want this to be driven from a notify on the individual events.
#[get("/api/poll")]
pub async fn poll(config: &State<Config>, conn: Database) -> Result<(), Error> {
    let client = reqwest::Client::new();

    let events: MailgunEvents = client
        .get("https://api.eu.mailgun.net/v3/tweede.golf/events?event=stored")
        .basic_auth("api", config.mailgun_key.as_ref())
        .send()
        .await?
        .json::<MailgunEvents>()
        .await?;

    for event in &events.items {
        process_message(config, &conn, &event.storage.url).await?;
    }

    Ok(())
}
