#[macro_use]
extern crate rocket;
extern crate dotenv;
extern crate rocket_sync_db_pools;

mod config;
mod email;
mod error;
mod id;
mod receive;
mod sentry;
mod sign;
mod storage;

use common::{DownloadResult, MessageData, SealedMessage};
use dotenv::dotenv;

use rocket::fairing::AdHoc;
use rocket::serde::json::Json;
use rocket::State;
use rocket_sync_db_pools::{database, postgres};
use sentry::SentryLogger;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::config::Config;
use crate::email::send_email;
use crate::error::Error;
use crate::id::Id;
use crate::receive::new_email;
#[cfg(debug_assertions)]
use crate::receive::poll;
use crate::sentry::SentryFairing;
use crate::sign::{sign_message, sign_result};

#[derive(Serialize, Deserialize, Debug)]
struct PostResult {
    error: bool,
    id: Option<String>,
}

fn encode_sealed_message(message: &SealedMessage) -> Result<Vec<u8>, Error> {
    Ok(serde_json::to_vec(message)?)
}

fn uses_allowed_attributes(config: &Config, message: &SealedMessage) -> bool {
    for attribute in &message.attributes {
        if !config.allowed_attributes.contains(&attribute.identifier.0) {
            return false;
        }
    }
    true
}

#[post("/api", format = "json", data = "<request>")]
async fn api(
    config: &State<Config>,
    conn: Database,
    request: Json<MessageData>,
) -> Result<(), Error> {
    request.validate()?;

    // Check whether attributes are allowed and files are not too big
    for message in &request.recipient_messages {
        if !uses_allowed_attributes(config, &message.sealed) {
            return Err(Error::InvalidAttribute);
        }
        if message.sealed.ct.len() > config.maximum_file_size {
            return Err(Error::TooBig);
        }
    }

    for message in &request.recipient_messages {
        let id = Id::new();
        let row_id = id.to_string();
        let from = request.from.clone();
        let subject = request.subject.clone();
        let to_copy = message.to.clone();
        let signature = request.signature.clone();

        conn.run(move |c| {
            c.execute(
                "INSERT INTO messages (id, from_address, to_address, subject, signature) VALUES ($1, $2, $3, $4, $5)",
                &[&row_id, &from, &to_copy, &subject, &signature],
            )
        }).await?;

        let encoded_message = encode_sealed_message(&message.sealed)?;
        config
            .storage
            .store(encoded_message.clone(), &id.to_string())
            .await?;

        send_email(
            config,
            &id,
            &request.from,
            &message.to,
            &request.subject,
            encoded_message,
        )?;
    }

    Ok(())
}

#[get("/api/storage/<slug>")]
async fn serve_storage(config: &State<Config>, slug: String) -> Result<Vec<u8>, Error> {
    config.storage.serve(&slug).await
}

#[get("/api/download/<id>")]
async fn download(
    config: &State<Config>,
    conn: Database,
    id: Id,
) -> Result<Json<DownloadResult>, Error> {
    let id_clone = id.clone();
    if let Some((from, to, subject, signature)) = conn
        .run(move |c| {
            let result = c
                .query(
                    "SELECT from_address, to_address, subject, signature FROM messages WHERE id = $1",
                    &[&id.to_string()],
                )
                .unwrap();
            result.get(0).map(|row| {
                (
                    row.get::<_, String>(0),
                    row.get::<_, String>(1),
                    row.get::<_, String>(2),
                    row.get::<_, Option<String>>(3),
                )
            })
        })
        .await
    {
        Ok(Json(DownloadResult {
            id: id_clone.to_string(),
            from,
            to,
            subject,
            signature,
            content: config.storage.retrieve_url(&id_clone.to_string()).await?,
        }))
    } else {
        Err(Error::NotFound)
    }
}

#[database("db")]
pub struct Database(postgres::Client);

#[launch]
fn boot() -> _ {
    dotenv().ok();
    SentryLogger::init();

    let base = setup(rocket::build());

    let config = base.figment().extract::<Config>().unwrap_or_else(|_| {
        // Ignore error value, as it could contain private keys
        log::error!("Failure to parse configuration");
        panic!("Failure to parse configuration")
    });

    match config.sentry_dsn {
        Some(dsn) => base.attach(SentryFairing::new(&dsn, "tguard backend")),
        None => base,
    }
}

fn setup(rocket: rocket::Rocket<rocket::Build>) -> rocket::Rocket<rocket::Build> {
    let rocket = rocket
        .attach(Database::fairing())
        .attach(AdHoc::config::<Config>())
        .mount(
            "/",
            routes![
                api,
                download,
                new_email,
                serve_storage,
                sign_message,
                sign_result
            ],
        );

    #[cfg(debug_assertions)]
    let rocket = rocket.mount("/", routes![poll]);

    rocket
}

#[cfg(test)]
mod test {
    use super::{rocket, setup, DownloadResult, SealedMessage};
    use cloud_storage::ListRequest;
    use cloud_storage::Object;
    use common::{AttributeIdentifier, AttributeValue};
    use figment::providers::Format;
    use figment::providers::Toml;
    use figment::Figment;
    use rocket::http::ContentType;
    use rocket::http::Status;
    use rocket::local::blocking::Client;
    use serde::Deserialize;
    use serde_json::json;
    use serial_test::serial;

    #[post("/setup_db")]
    async fn setup_db(conn: super::Database) {
        conn.run(|c| {
            c.batch_execute(include_str!("../db.sql")).unwrap();
        })
        .await;
    }

    // Clear the mailhog message list
    fn reset_mailhog(mailhog_host: &str) {
        let client = reqwest::blocking::Client::new();
        client
            .delete(format!("http://{}:1080/api/v1/messages", mailhog_host))
            .send()
            .expect("Failed to reset mailhog message list")
            .error_for_status()
            .expect("Failed to reset mailhog message list");
    }

    // Clear the object bucket
    fn reset_bucket() {
        for sublist in Object::list_sync("tguard_test", ListRequest::default()).unwrap() {
            for object in sublist.items {
                Object::delete_sync("tguard_test", &object.name).unwrap();
            }
        }
    }

    // Fetch object
    fn fetch_object(url: String) -> SealedMessage {
        let client = reqwest::blocking::Client::new();
        client
            .get(url)
            .send()
            .expect("Failed to retrieve object")
            .error_for_status()
            .expect("Failed to retrieve object")
            .json()
            .expect("Invalid message")
    }

    #[derive(Deserialize)]
    struct MailhogMessageRaw {
        #[serde(rename = "Data")]
        data: String,
    }

    #[derive(Deserialize)]
    struct MailhogMessage {
        #[serde(rename = "Raw")]
        raw: MailhogMessageRaw,
    }

    // Extract id from the first email in the message list provided by mailhog
    fn extract_id_from_mailhog(mailhog_host: &str) -> String {
        let client = reqwest::blocking::Client::new();
        let messages: Vec<MailhogMessage> = client
            .get(format!("http://{}:1080/api/v1/messages", mailhog_host))
            .send()
            .expect("Failed to fetch messages")
            .json()
            .expect("Failed to parse mailhog result");
        let message = messages.get(0).expect("No email received");
        let mut parts = message.raw.data.split("https://example.com/download/");
        parts.next().expect("Mail content incomplete");
        let part = parts.next().expect("Mail content incomplete");
        let mut parts = part.split(' ');
        parts.next().expect("Mail content incomplete").into()
    }

    #[test]
    #[serial]
    fn api() {
        // Setup
        let mailhog_host = option_env!("MAILHOG_HOST").expect("Missing Mailhog host");
        let postgres_url = option_env!("TEST_DB").expect("Missing test database");
        let figment = Figment::from(rocket::config::Config::default())
            .select(rocket::Config::DEFAULT_PROFILE)
            .merge(Toml::string(&format!(
                r#"
host = "https://example.com"
mail_user = "test@example.com"
mail_host = "{}"
mail_port = 1025
mailgun_message_url_prefix = "https://storage.eu.mailgun.net/v3/domains/tweede.golf/messages/"
from_fallback = "test@example.com"
storage_type = "gcs"
storage_location = "tguard_test"
allowed_attributes = ["pbdf.sidn-pbdf.email.email"]
allowed_signing_attributes = ["pbdf.sidn-pbdf.email.email"]
irmaserver = "http://127.0.0.1:8088"
maximum_file_size = 32767

[databases]
db = {{ url = "{}" }}
                "#,
                mailhog_host, postgres_url
            )));
        let client = Client::tracked(setup(rocket::custom(figment)).mount("/", routes![setup_db]))
            .expect("valid rocket instance");
        assert_eq!(client.post("/setup_db").dispatch().status(), Status::Ok);
        reset_mailhog(mailhog_host);
        reset_bucket();

        let response = client.post("/api")
            .header(ContentType::JSON)
            .body(json!({
                "from": "from@example.com",
                "subject": "Example subject",
                "recipient_messages": [
                    {
                        "to": "to@example.com",
                        "sealed": {
                            "c_key": "h9J6WdqlnSgHEULkJbDJ1zBKjJ+LAWaTqEwlAUG5gA9GHT0S3I+0emOES7nfdzpOCEGqbfdDffMEFwqEiW7wGyR3NZJxSmM3GYwTJdZqNbTHosucrw+MsYctOdWdXHS9rfdQBtvlqUE1xYbCnrjsN4RHMpyUj2H+yHit70d0re5CIxUp0yArdidBz6LjUPpd",
                            "ct": "gAMMKLikymhNIDeqUjqjJqEFTj8qWnrUUUhwCrIG6sOplxR4pFnUKA==",
                            "iv": "0z6La7O6CfxcvND0LqDQBA==",
                            "timestamp": 1629883307061_u64,
                            "attributes": [
                                {
                                    "identifier": AttributeIdentifier("pbdf.sidn-pbdf.email.email".to_owned()),
                                    "value": "to@example.com",
                                },
                            ],
                        }
                    },
                ],
            }).to_string())
            .dispatch();

        assert_eq!(response.status(), Status::Ok);

        let id = extract_id_from_mailhog(mailhog_host);

        let response = client.get(format!("/api/download/{}", &id)).dispatch();

        assert_eq!(response.status(), Status::Ok);

        let response_body = response.into_string().unwrap();
        let result: DownloadResult = serde_json::from_str(&response_body).unwrap();

        let expected = DownloadResult {
            id,
            to: "to@example.com".to_string(),
            from: "from@example.com".to_string(),
            subject: "Example subject".to_string(),
            signature: None,
            content: result.content.clone(),
        };

        assert_eq!(result, expected);

        let message = fetch_object(result.content);

        let expected_message = SealedMessage {
            c_key: "h9J6WdqlnSgHEULkJbDJ1zBKjJ+LAWaTqEwlAUG5gA9GHT0S3I+0emOES7nfdzpOCEGqbfdDffMEFwqEiW7wGyR3NZJxSmM3GYwTJdZqNbTHosucrw+MsYctOdWdXHS9rfdQBtvlqUE1xYbCnrjsN4RHMpyUj2H+yHit70d0re5CIxUp0yArdidBz6LjUPpd".to_string(),
            ct: "gAMMKLikymhNIDeqUjqjJqEFTj8qWnrUUUhwCrIG6sOplxR4pFnUKA==".to_string(),
            iv: "0z6La7O6CfxcvND0LqDQBA==".to_string(),
            timestamp: 1629883307061,
            attributes: vec![AttributeValue {
                identifier: AttributeIdentifier("pbdf.sidn-pbdf.email.email".to_owned()),
                value: "to@example.com".into()
            }],
        };

        assert_eq!(message, expected_message);
    }
}
