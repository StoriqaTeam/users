use std::string::ToString;

/// Status Message - generic information status response
#[derive(Serialize, Deserialize, Debug)]
pub struct StatusMessage {
    pub status: String
}

impl StatusMessage {
    /// Creates new `StatusMessage` from string literal
    pub fn new(msg: &str) -> StatusMessage {
        StatusMessage {
            status: msg.to_string()
        }
    }
}

impl ToString for StatusMessage {
    fn to_string(&self) -> String {
        self.status.clone()
    }
}
