use base64::{decode, encode};
use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::query_dsl::RunQueryDsl;
use diesel::Connection;
use r2d2::ManageConnection;
use rand;
use rand::Rng;
use services::Service;
use sha3::{Digest, Sha3_256};

use super::types::ServiceFuture;
use errors::Error;
use repos::repo_factory::ReposFactory;
use repos::types::RepoResult;

pub trait UtilService {
    fn clear_database(&self) -> ServiceFuture<()>;
}

impl<
        T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
        M: ManageConnection<Connection = T>,
        F: ReposFactory<T>,
    > UtilService for Service<T, M, F>
{
    fn clear_database(&self) -> ServiceFuture<()> {
        debug!("Truncating tables");
        self.spawn_on_pool(|conn| {
            let _ = diesel::sql_query("TRUNCATE TABLE identities, reset_tokens, user_roles, users;")
                .execute(&*conn)?;
            let _ = diesel::sql_query("INSERT INTO users (id, email, last_login_at, saga_id) VALUES (1, 'admin@storiqa.com', now(), 'a4cb84cb-62a7-45c6-939e-7c57cc399d5a') ON CONFLICT (id) DO NOTHING;")
                .execute(&*conn)?;
            let _ = diesel::sql_query("INSERT INTO identities (user_id, email, provider, password, saga_id) SELECT id, email, '', 'ivcHmQPHBx9EUGql4Zv8EaXCkQcswPuL905JCp5ss5k=.js5QVSk6FG', 'a4cb84cb-62a7-45c6-939e-7c57cc399d5a' FROM users WHERE email = 'admin@storiqa.com' LIMIT 1;")
                .execute(&*conn)?;
            let _ = diesel::sql_query("INSERT INTO user_roles (user_id, name) SELECT id, 'superuser' FROM users WHERE email = 'admin@storiqa.com' LIMIT 1;")
                .execute(&*conn)?;

            Ok(())
        })
    }
}

pub fn password_create(clear_password: String) -> String {
    let salt = rand::thread_rng().gen_ascii_chars().take(10).collect::<String>();
    let pass = clear_password + &salt;
    let mut hasher = Sha3_256::default();
    hasher.input(pass.as_bytes());
    let out = hasher.result();
    let computed_hash = encode(&out[..]);
    computed_hash + "." + &salt
}

pub fn password_verify(db_hash: &str, clear_password: String) -> RepoResult<bool> {
    let v: Vec<&str> = db_hash.split('.').collect();
    if v.len() != 2 {
        Err(Error::Validate(validation_errors!({"password": ["password" => "Password in db has wrong format"]})).into())
    } else {
        let salt = v[1];
        let pass = clear_password + salt;
        let mut hasher = Sha3_256::default();
        hasher.input(pass.as_bytes());
        let out = hasher.result();
        decode(v[0])
            .map(|computed_hash| computed_hash == &out[..])
            .map_err(|_| Error::Validate(validation_errors!({"password": ["password" => "Password in db has wrong format"]})).into())
    }
}
