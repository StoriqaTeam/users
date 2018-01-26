#[derive(PartialEq, Eq)]
pub enum Scope {
    All,
    Owned,
}

impl Scope {
    pub fn can(&self, user_id: Option<i32>, resource_owner_id: Option<i32>) -> bool {
        match (user_id, resource_owner_id, self) {
            (Some(user_id), Some(resource_owner_id), &Scope::Owned) =>
                user_id == resource_owner_id,

            (_, _, &Scope::All) => true,

            _ => false
        }
    }
}
