use std::fmt::{Display, Formatter};

use rand::distributions::Alphanumeric;
use rand::Rng;
use rocket::request::FromParam;

#[derive(Clone, Debug)]
pub struct Id(String);

impl Id {
    pub fn new() -> Self {
        Id(rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect())
    }
}

impl<'a> FromParam<'a> for Id {
    type Error = &'static str;

    fn from_param(param: &'a str) -> Result<Self, Self::Error> {
        if param.len() == 32 && param.chars().all(char::is_alphanumeric) {
            Ok(Id(param.to_owned()))
        } else {
            Err("invalid id")
        }
    }
}

impl Display for Id {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
