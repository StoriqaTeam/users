use base64::{encode, decode};
use rand;
use rand::Rng;
use sha3::{Digest, Sha3_256};

use services::error::ServiceError;

pub fn password_create(clear_password: String) -> String {
    let salt = rand::thread_rng().gen_ascii_chars().take(10).collect::<String>();
    let pass = clear_password + &salt;
    let mut hasher = Sha3_256::default();
    hasher.input(pass.as_bytes());
    let out = hasher.result();
    let computed_hash = encode(&out[..]);
    computed_hash + "." + &salt
}

pub fn password_verify(db_hash: String, clear_password: String) -> Result<bool, ServiceError> {
    let v: Vec<&str> = db_hash.split('.').collect();
    if v.len() != 2 {
        Err(ServiceError::Validate(
            validation_errors!({"password": ["password" => "Password in db has wrong format"]}),
        ))
    } else {
        let salt = v[1];
        let pass = clear_password + salt;
        let mut hasher = Sha3_256::default();
        hasher.input(pass.as_bytes());
        let out = hasher.result();
        let computed_hash = decode(v[0])
            .map_err(|_| ServiceError::Validate(validation_errors!({"password": ["password" => "Password in db has wrong format"]})))?;
        Ok(computed_hash == &out[..])
    }
}
