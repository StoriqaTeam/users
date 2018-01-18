use std::env;
use config::{Config, ConfigError, Environment, File};

/// Basic settings - HTTP binding address and database DSN
#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub server: Server,    
    pub client: Client,
    pub jwt: JWT,
    pub google: OAuth,
    pub facebook: OAuth,
}

/// Common server settings
#[derive(Debug, Deserialize, Clone)]
pub struct Server {
    pub address: String,
    pub database: String,
    pub thread_count: usize,
}

/// Http client settings
#[derive(Debug, Deserialize, Clone)]
pub struct Client {
    pub http_client_retries: usize,
    pub http_client_buffer_size: usize,
    pub dns_worker_thread_count: usize
}

/// Json Web Token seettings
#[derive(Debug, Deserialize, Clone)]
pub struct JWT {
    pub secret_key: String,
}

/// Oauth 2.0 basic settings
#[derive(Debug, Deserialize, Clone)]
pub struct OAuth {
    pub id: String,
    pub key: String,
    pub info_url: String,
    pub redirect_url: String,
    pub code_to_token_url: String,
}


impl Settings {
    /// Creates new app settings struct
    /// #Examples
    /// ```
    /// use users_lib::settings::*;
    /// 
    /// let settings = Settings::new();
    /// ```
    pub fn new() -> Result<Self, ConfigError> {
        let mut s = Config::new();
        s.merge(File::with_name("config/base"))?;

        // Note that this file is _optional_
        let env = env::var("RUN_MODE").unwrap_or("development".into());
        s.merge(File::with_name(&format!("config/{}", env)).required(false))?;

        // Add in settings from the environment (with a prefix of STQ_USERS)
        s.merge(Environment::with_prefix("STQ_USERS"))?;

        s.try_into()
    }
}