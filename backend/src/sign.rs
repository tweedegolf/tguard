use std::collections::HashMap;

use common::SignResult;
use irma::{AttributeRequest, ProofStatus, SessionData, SessionToken, SignatureRequestBuilder};
use rocket::{serde::json::Json, State};
use serde::{Deserialize, Serialize};

use crate::{config::Config, error::Error};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SignRequest {
    hash: String,
    attributes: Vec<String>,
}

#[post("/api/sign", data = "<request>")]
pub async fn sign_message(
    config: &State<Config>,
    request: Json<SignRequest>,
) -> Result<Json<SessionData>, Error> {
    let mut sig_builder =
        SignatureRequestBuilder::new(format!("Tguard bericht met hash {}", request.hash));
    for attr in &request.attributes {
        if config.allowed_signing_attributes.contains(attr) {
            sig_builder =
                sig_builder.add_discon(vec![vec![AttributeRequest::Simple(attr.to_owned())]])
        } else {
            return Err(Error::InvalidAttribute);
        }
    }
    Ok(Json(config.irmaserver.request(&sig_builder.build()).await?))
}

#[get("/api/sign_result?<session>")]
pub async fn sign_result(
    config: &State<Config>,
    session: String,
) -> Result<Json<SignResult>, Error> {
    let result = config.irmaserver.result(&SessionToken(session)).await?;
    if let Some(proof_status) = result.proof_status {
        if proof_status != ProofStatus::Valid {
            return Err(Error::NotFound);
        }
        if let Some(signature) = result.signature {
            let mut attributes = HashMap::new();
            for attr_list in result.disclosed {
                for attr in attr_list {
                    if let Some(val) = attr.raw_value {
                        attributes.insert(attr.identifier, val);
                    }
                }
            }

            Ok(Json(SignResult {
                signature,
                attributes,
            }))
        } else {
            Err(Error::NotFound)
        }
    } else {
        Err(Error::NotFound)
    }
}
