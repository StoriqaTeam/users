//! Enum for scopes available in ACLs

#[derive(PartialEq, Eq)]
pub enum Scope {
    /// Resource with any id
    All,

    /// Resource with id of the owner equal to the id of the current user
    Owned,
}
