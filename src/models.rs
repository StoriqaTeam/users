//use serde::ser::{self, Serialize, Serializer};

#[derive(Debug, Serialize, Queryable, Deserialize)]
pub struct User {
    pub id: i32,
    pub email: String,
    pub password: String,
    pub is_active: bool,
}

/*
// Serialization failure test
impl Serialize for User {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        Err(ser::Error::custom("path contains invalid UTF-8 characters"))
    }
}
*/
