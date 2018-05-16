//! Enum for resources available in ACLs
use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Resource {
    Users,
    UserRoles,
    UserDeliveryAddresses,
}

impl fmt::Display for Resource {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Resource::Users => write!(f, "users"),
            Resource::UserRoles => write!(f, "user roles"),
            Resource::UserDeliveryAddresses => write!(f, "user delivery addresses"),
        }
    }
}
