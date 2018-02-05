//! Action enum for authorization

// All gives all permissions.
// Index - list resources, Read - read resource with id,
// Write - Update or delete resource with id.
#[derive(PartialEq, Eq)]
pub enum Action {
    All,
    Read,
    Write,
}
