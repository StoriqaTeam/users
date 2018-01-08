use payloads::jwt::ProviderOauth;
use payloads::user::NewUser;
use models::jwt::JWT;
use jsonwebtoken::{encode, Header, Algorithm};
use error::Error as ApiError;


/// JWT repository, responsible for handling jwt
pub struct JWTRepo {
    pub secret_key: String,
}


impl JWTRepo {
    /// Creates JWT for user
    pub fn create_token_user(&self, user: NewUser) -> Result<JWT, ApiError> {
        let token = encode(&Header::default(), &user, self.secret_key.as_ref())?;
        Ok (JWT { token: token})
    }

    /// Creates JWT for user with google oauth
    pub fn create_token_google(&self, oauth: ProviderOauth) -> Result<JWT, ApiError> {
        let token = encode(&Header::default(), &oauth, self.secret_key.as_ref())?;
        Ok (JWT { token: token})
    }

    /// Creates JWT for user with facebook oauth
    pub fn create_token_facebook(&self, oauth: ProviderOauth) -> Result<JWT, ApiError> {
        let token = encode(&Header::default(), &oauth, self.secret_key.as_ref())?;
        Ok (JWT { token: token})
    }
    
}
