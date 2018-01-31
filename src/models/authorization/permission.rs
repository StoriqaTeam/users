//! Permission is a tuple for describing permisssions

use models::{Resource, Action, Scope};

pub struct Permission {
    pub resource: Resource,
    pub action: Action,
    pub scope: Scope,
}
