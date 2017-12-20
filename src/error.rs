use serde_json;
use diesel;

pub enum Error {
    Default(JsonError),
    Json(serde_json::error::Error),
    Database(diesel::result::Error)
}

impl Error {
    pub fn new(message: &str) -> Error {
        let json_error = JsonError { error: message.into() };
        Error::Default(json_error)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonError {
    error: String
}

impl JsonError {
    pub fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}
