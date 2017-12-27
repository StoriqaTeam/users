/// User model (WIP)
#[derive(Debug, Serialize, Queryable, Deserialize)]
pub struct User {
    pub id: i32,
    pub email: String,
    pub password: String,
    pub is_active: bool,
}
