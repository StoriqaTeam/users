use std::env;
use config::{Config, ConfigError, Environment, File};

#[derive(Debug, Deserialize, Clone)]
pub struct Http {
    pub bind: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Database {
    pub dsn: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub http: Http,
    pub database: Database,
}

impl Settings {
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
