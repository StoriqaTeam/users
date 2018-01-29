//! Enum for resources available in ACLs

#[derive(PartialEq, Eq)]
pub enum Resource {
    Users,
    UserRoles,
}
