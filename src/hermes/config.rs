use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// Telegram bot token
    pub token: String,

    /// Telegram chat id
    pub chat_id: i64,
}

pub fn load_config() -> Result<Config> {
    let contents =
        std::fs::read_to_string("/etc/hermes/config.toml").context("Failed to read config file")?;

    toml::from_str(&contents).context("Failed to parse config file")
}
