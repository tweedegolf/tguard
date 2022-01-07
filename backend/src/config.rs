use crate::storage::{CloudStorage, LocalStorage, Storage};
use std::{collections::HashSet, convert::TryFrom, iter::FromIterator};

use irma::{IrmaClient, IrmaClientBuilder};
use lettre::{message::Mailbox, transport::smtp::authentication::Credentials, SmtpTransport};
use log::warn;
use serde::Deserialize;

#[derive(Deserialize)]
enum StorageType {
    #[serde(rename = "local")]
    Local,
    #[serde(rename = "gcs")]
    Gcs,
}

#[derive(Deserialize)]
struct RawConfig {
    host: String,
    mail_user: String,
    mail_pass: Option<String>,
    mail_host: String,
    mail_port: u16,
    sentry_dsn: Option<String>,
    mailgun_key: Option<String>,
    mailgun_message_url_prefix: String,
    from_fallback: String,
    storage_type: StorageType,
    storage_location: String,
    allowed_attributes: Vec<String>,
    allowed_signing_attributes: Vec<String>,
    irmaserver: String,
    irmaserver_token: Option<String>,
    maximum_file_size: usize,
}

#[derive(Deserialize)]
#[serde(try_from = "RawConfig")]
pub struct Config {
    pub host: String,
    pub mail_user: Mailbox,
    pub mailer: SmtpTransport,
    pub sentry_dsn: Option<String>,
    pub mailgun_key: Option<String>,
    pub mailgun_message_url_prefix: String,
    pub from_fallback: String,
    pub storage: Box<dyn Storage>,
    pub allowed_attributes: HashSet<String>,
    pub allowed_signing_attributes: HashSet<String>,
    pub irmaserver: IrmaClient,
    pub maximum_file_size: usize,
}

impl TryFrom<RawConfig> for Config {
    type Error = crate::error::Error;

    fn try_from(v: RawConfig) -> Result<Self, Self::Error> {
        let mailer = match v.mail_pass {
            Some(pass) => {
                let credentials = Credentials::new(v.mail_user.clone(), pass);
                SmtpTransport::starttls_relay(&v.mail_host)?
                    .port(v.mail_port)
                    .credentials(credentials)
                    .build()
            }
            None => {
                warn!("Warning: No email password specified, using insecure mailer");
                SmtpTransport::builder_dangerous(v.mail_host)
                    .port(v.mail_port)
                    .build()
            }
        };

        let irmaserver = match v.irmaserver_token {
            Some(token) => IrmaClientBuilder::new(&v.irmaserver)?
                .token_authentication(token)
                .build(),
            None => IrmaClient::new(&v.irmaserver)?,
        };

        let mail_user: Mailbox = v.mail_user.parse()?;
        Ok(Config {
            storage: match v.storage_type {
                StorageType::Local => Box::new(LocalStorage {
                    host: v.host.clone(),
                    directory: v.storage_location,
                }),
                StorageType::Gcs => Box::new(CloudStorage {
                    bucket: v.storage_location,
                }),
            },
            host: v.host,
            mail_user,
            mailer,
            sentry_dsn: v.sentry_dsn,
            mailgun_key: v.mailgun_key,
            mailgun_message_url_prefix: v.mailgun_message_url_prefix,
            from_fallback: v.from_fallback,
            allowed_attributes: HashSet::from_iter(v.allowed_attributes.into_iter()),
            allowed_signing_attributes: HashSet::from_iter(
                v.allowed_signing_attributes.into_iter(),
            ),
            irmaserver,
            maximum_file_size: v.maximum_file_size,
        })
    }
}
