use crate::{Error, Result};
use std::{env, str::FromStr, sync::OnceLock};

// config is small enough to use the crate errors instead of using its own errors

/// Retrieves the global configuration.
///
/// This function lazily initializes and returns a reference to the global `Config` object.
pub fn config() -> &'static Config {
    // only one thread can acquire the lock and initialize the config
    static INSTANCE: OnceLock<Config> = OnceLock::new();

    // here we want to panic because if the config is not there, we don't want the app
    // to start at all
    INSTANCE.get_or_init(|| {
        Config::load_from_env()
            .unwrap_or_else(|ex| panic!("FATAL - WHILE LOADING CONF - Cause: {ex}"))
    })
}

#[allow(non_snake_case)]
pub struct Config {
    // -- Crypt
    pub PWD_KEY: Vec<u8>,
    pub TOKEN_KEY: Vec<u8>,
    pub TOKEN_DURATION_SEC: f64,
    // -- DB
    pub DB_URL: String,
    // -- Web
    pub WEB_FOLDER: String,
}

impl Config {
    /// we could load the env variables with dotenv
    /// but in a production scenario, it is better that these variables are
    /// set beforehand. This is why we use the cargo config for dev
    fn load_from_env() -> Result<Config> {
        Ok(Config {
            PWD_KEY: get_env_b64u_as_u8s("SERVICE_PWD_KEY")?,
            TOKEN_KEY: get_env_b64u_as_u8s("SERVICE_TOKEN_KEY")?,
            TOKEN_DURATION_SEC: get_env_parse("SERVICE_TOKEN_DURATION_SEC")?,
            DB_URL: get_env("SERVICE_DB_URL")?,
            WEB_FOLDER: get_env("SERVICE_WEB_FOLDER")?,
        })
    }
}

fn get_env(name: &'static str) -> Result<String> {
    env::var(name).map_err(|_| Error::ConfigMissingEnv(name))
}

fn get_env_b64u_as_u8s(name: &'static str) -> Result<Vec<u8>> {
    base64_url::decode(&get_env(name)?).map_err(|_| Error::ConfigWrongFormat(name))
}

fn get_env_parse<T: FromStr>(name: &'static str) -> Result<T> {
    let val = get_env(name)?;
    val.parse::<T>().map_err(|_| Error::ConfigWrongFormat(name))
}
