use super::Scope;

pub trait WithScope {
    fn is_in_scope(&self, scope: &Scope, user_id: i32) -> bool;
}
