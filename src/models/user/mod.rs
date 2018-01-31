//! Models for working with users

pub mod user;
pub mod gender;

pub use self::user::User;
pub use self::user::NewUser;
pub use self::user::UpdateUser;
pub use self::gender::Gender;
