//! Models for working with identities

pub mod identity;
pub mod provider;

pub use self::identity::Identity;
pub use self::identity::NewIdentity;
pub use self::identity::NewEmailIdentity;
pub use self::provider::Provider;
