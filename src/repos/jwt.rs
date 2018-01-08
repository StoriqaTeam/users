use payloads::jwt::ProviderOauth;
use payloads::user::NewUser;
use models::jwt::JWT;
use frank_jwt::{Header, Payload, Algorithm, encode, decode};

/// JWT repository, responsible for handling jwt
pub struct JWTRepo {
    pub secret_key: String,
}


impl JWTRepo {
    /// Creates JWT for user
    pub fn create_token_user(&self, user: NewUser) -> JWT {
        let mut payload = Payload::new();
        payload.insert("email".to_string(), user.email.to_string());
        payload.insert("password".to_string(), user.password.to_string());
        let header = Header::new(Algorithm::HS256);
        let token = encode(header, self.secret_key.to_string(), payload.clone());
        JWT { token: token}
    }

/// Creates JWT for user
    pub fn create_token_google(&self, oauth: ProviderOauth) -> JWT {
        let mut payload = Payload::new();
        payload.insert("token".to_string(), oauth.token);
        payload.insert("provider".to_string(), "google".to_string());
        let header = Header::new(Algorithm::HS256);
        let token = encode(header, self.secret_key.to_string(), payload.clone());
        JWT { token: token}
    }

    /// Creates JWT for user
    pub fn create_token_facebook(&self, oauth: ProviderOauth) -> JWT {
        let mut payload = Payload::new();
        payload.insert("token".to_string(), oauth.token);
        payload.insert("provider".to_string(), "facebook".to_string());
        let header = Header::new(Algorithm::HS256);
        let token = encode(header, self.secret_key.to_string(), payload.clone());
        JWT { token: token}
    }


    
}
