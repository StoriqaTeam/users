use schema::users;
//use serde::ser::{self, Serialize, Serializer};

#[derive(Debug, Serialize, Queryable, Deserialize)]
pub struct User {
    pub id: i32,
    pub email: String,
    pub password: String,
    pub is_active: bool,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[table_name = "users"]
pub struct NewUser<'a> {
    pub email: &'a str,
    pub password: &'a str,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[table_name = "users"]
pub struct UpdateUser<'a> {
    pub email: &'a str
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
