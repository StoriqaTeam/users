//! Models contains all structures that are used in different
//! modules of the app

pub mod authorization;
pub mod identity;
pub mod jwt;
pub mod reset_token;
pub mod user;
pub mod user_role;

pub use self::authorization::*;
pub use self::identity::*;
pub use self::jwt::*;
pub use self::reset_token::*;
pub use self::user::*;
pub use self::user_role::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SagaCreateProfile {
    pub user: Option<NewUser>,
    pub identity: NewIdentity,
}
