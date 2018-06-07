use base64::{decode, encode};
use rand;
use rand::Rng;
use sha3::{Digest, Sha3_256};

use errors::ControllerError;
use repos::types::RepoResult;

pub fn password_create(clear_password: String) -> String {
    let salt = rand::thread_rng().gen_ascii_chars().take(10).collect::<String>();
    let pass = clear_password + &salt;
    let mut hasher = Sha3_256::default();
    hasher.input(pass.as_bytes());
    let out = hasher.result();
    let computed_hash = encode(&out[..]);
    computed_hash + "." + &salt
}

pub fn password_verify(db_hash: String, clear_password: String) -> RepoResult<bool> {
    let v: Vec<&str> = db_hash.split('.').collect();
    if v.len() != 2 {
        Err(ControllerError::Validate(validation_errors!({"password": ["password" => "Password in db has wrong format"]})).into())
    } else {
        let salt = v[1];
        let pass = clear_password + salt;
        let mut hasher = Sha3_256::default();
        hasher.input(pass.as_bytes());
        let out = hasher.result();
        decode(v[0]).map(|computed_hash| computed_hash == &out[..]).map_err(|_| {
            ControllerError::Validate(validation_errors!({"password": ["password" => "Password in db has wrong format"]})).into()
        })
    }
}
