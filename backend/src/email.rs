use crate::id::Id;
use crate::Config;
use crate::Error;

use lettre::{
    message::{
        header::{ContentDisposition, ContentTransferEncoding},
        MaybeString, MultiPart, SinglePart, SinglePartBuilder,
    },
    Message, Transport,
};

pub fn send_email(
    config: &Config,
    id: &Id,
    from: &str,
    to: &str,
    subject: &str,
    body: Vec<u8>,
) -> Result<(), Error> {
    let url = format!("{}/download/{}", config.host, id);
    let text_body = include_str!("message.txt")
        .to_string()
        .replace(":from", from)
        .replace(":url", &url);

    let seal = SinglePartBuilder::new()
        .content_type(
            "application/irmaseal; name=\"irmaseal.encrypted\""
                .parse()
                .unwrap(),
        )
        .header(ContentDisposition::attachment("encrypted.irmaseal"))
        .header(ContentTransferEncoding::Base64)
        .body(MaybeString::Binary(body));

    let text = SinglePart::plain(text_body);

    // let html = SinglePart::html(html_body);
    // let fallback = MultiPart::alternative_plain_html(text, html);

    let fallback = text;

    let content = MultiPart::mixed().singlepart(seal).singlepart(fallback);

    let email = Message::builder()
        .from(config.mail_user.clone())
        .reply_to(from.parse()?)
        .to(to.parse()?)
        .subject(subject)
        .multipart(content)
        .unwrap();

    config.mailer.send(&email)?;
    Ok(())
}

pub fn send_confirmation_email(
    config: &Config,
    id: Id,
    to: &str,
    subject: &str,
) -> Result<(), Error> {
    let url = format!("{}/download/{}", config.host, id);
    let body = include_str!("confirm_message.txt")
        .to_string()
        .replace(":url", &url);

    let email = Message::builder()
        .from(config.mail_user.clone())
        .to(to.parse()?)
        .subject(format!("Re: {}", subject))
        .body(body)?;

    config.mailer.send(&email)?;

    Ok(())
}
