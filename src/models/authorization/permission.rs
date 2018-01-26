use super::{Resource, Action, Scope};

pub struct Permission {
    pub resource: Resource,
    pub action: Action,
    pub scope: Scope,
}
