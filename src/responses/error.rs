use error::Error;

/// ErrorMessage - interop structure to serialize `Error` into JSON
#[derive(Serialize, Deserialize, Debug)]
pub struct ErrorMessage {
    pub code: u16,
    pub message: String
}

impl ErrorMessage {
    /// Creates new `ErrorMessage` from `Error`
    pub fn new(error: &Error) -> ErrorMessage {
        ErrorMessage {
            code: error.to_code().as_u16(),
            message: error.to_string()
        }
    }
}
