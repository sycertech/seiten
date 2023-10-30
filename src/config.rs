use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

pub static CONFIG: Lazy<Config> = Lazy::new(|| Config::new().expect("Unable to retrieve config"));

/// Application Config
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    /// Which directories are allowed to be served
    pub directories: Vec<String>,
    /// The public GPG Key
    pub gpg_public_key: String,
    /// The port to listen on
    #[serde(default = "default_port")]
    pub port: u16,
}

fn default_port() -> u16 {
    63208
}

impl Config {
    /// Create a new `Config`
    pub fn new() -> anyhow::Result<Self> {
        let config = envy::from_env::<Self>()?;

        Ok(config)
    }
}

/// Get the default static `Config`
pub fn get_config() -> &'static Config {
    &CONFIG
}
