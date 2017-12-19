use schema::users;

#[derive(Serialize, Deserialize, Queryable)]
pub struct User {
    pub id: i32,
    pub email: String,
    pub password: String,
    pub is_active: bool,
}

#[derive(Deserialize, Insertable)]
#[table_name = "users"]
pub struct NewUser<'a> {
    pub email: &'a str,
    pub password: &'a str,
}

#[derive(Deserialize, Insertable)]
#[table_name = "users"]
pub struct UpdateUser<'a> {
    pub email: &'a str
}
