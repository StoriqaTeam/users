use super::Scope;

pub trait WithScope {
    fn in_scope(&self, scope: Scope, user_id: i32);
}
