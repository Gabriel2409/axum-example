use crate::{Error, Result};
use std::{env, sync::OnceLock};

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
    // -- Web
    pub WEB_FOLDER: String,
}

impl Config {
    /// we could load the env variables with dotenv
    /// but in a production scenario, it is better that these variables are
    /// set beforehand. This is why we use the cargo config for dev
    fn load_from_env() -> Result<Config> {
        Ok(Config {
            WEB_FOLDER: get_env("SERVICE_WEB_FOLDER")?,
        })
    }
}

fn get_env(name: &'static str) -> Result<String> {
    env::var(name).map_err(|_| Error::ConfigMissingEnv(name))
}
