//! Trait for telling if current resource is in scope

use super::Scope;

/// Implement this trait on resource to signal if it's in the current scope
pub trait WithScope {
    fn is_in_scope(&self, scope: &Scope, user_id: i32) -> bool;
}
