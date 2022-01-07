use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Mailing error: {0}")]
    EmailTransport(#[from] lettre::transport::smtp::Error),
    #[error("Mailing error: {0}")]
    EmailCompose(#[from] lettre::error::Error),
    #[error("Email address error: {0}")]
    EmailAddress(#[from] lettre::address::AddressError),
    #[error("Database error: {0}")]
    Database(#[from] postgres::Error),
    #[error("IRMA error: {0}")]
    Irma(#[from] irma::Error),
    #[error("Invalid request: {0}")]
    Validation(#[from] validator::ValidationErrors),
    #[error("Invalid attribute")]
    InvalidAttribute,
    #[error("Invalid API url")]
    InvalidApiUrl,
    #[error("Too big")]
    TooBig,
    #[error("Invalid encoding: {0}")]
    Encoding(#[from] serde_json::Error),
    #[error("Object storage: {0}")]
    Storage(#[from] cloud_storage::Error),
    #[error("Reqwest: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("File error: {0}")]
    File(#[from] std::io::Error),
    #[error("Not found")]
    NotFound,
    #[error("Missing data")]
    MissingData,
}

impl<'r, 'o: 'r> rocket::response::Responder<'r, 'o> for Error {
    fn respond_to(self, request: &'r rocket::Request<'_>) -> rocket::response::Result<'o> {
        match self {
            Error::NotFound => rocket::response::status::NotFound::<()>(()).respond_to(request),
            Error::InvalidAttribute => rocket::response::status::BadRequest::<&'static str>(Some(
                "Attribute used for encryption is not allowed",
            ))
            .respond_to(request),
            Error::TooBig => rocket::response::status::BadRequest::<&'static str>(Some(
                "Encrypted data too large",
            ))
            .respond_to(request),
            Error::Validation(e) => {
                rocket::response::status::BadRequest::<String>(Some(e.to_string()))
                    .respond_to(request)
            }
            _ => rocket::response::Debug::from(self).respond_to(request),
        }
    }
}
