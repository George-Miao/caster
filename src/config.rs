use std::path::PathBuf;

use color_eyre::{eyre::bail, Result};
use figment::{
    providers::{Env, Format, Toml},
    Figment,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Path of sled file
    #[serde(default = "default_db_path")]
    pub db_path: String,

    /// Size of broadcast size
    #[serde(default = "default_channel_size")]
    pub channel_size: usize,

    /// Log level. Value: ERROR, WARN, INFO, DEBUG, TRACE.
    #[serde(default = "default_log_level")]
    pub log_level: String,

    /// Config of feed caster
    pub caster_feed: Option<FeedConfig>,

    /// Config of crates.io caster
    pub caster_crates: Option<CratesConfig>,

    /// Config of telegram consumer
    pub consumer_telegram: Option<TelegramConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedConfig {
    pub urls: Vec<String>,

    /// How old a newly seen entry will be ignored
    #[serde(default = "default_feed_ignore_days")]
    pub ignore_days: u64,

    /// Interval between requests, in second
    #[serde(default = "default_feed_interval")]
    pub interval: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CratesConfig {
    pub crates: Vec<String>,

    /// Interval between updating crates.io index, in second (this will disable
    /// the cache so low interval may cause performance issue). It's suggested
    /// that you update every other minute.
    #[serde(default = "default_feed_interval")]
    pub interval: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramConfig {
    pub api_token: String,
    pub chats: Vec<i64>,

    /// Max text length of content
    #[serde(default = "default_telegram_content_max_length")]
    pub content_max_length: usize,
}

impl Config {
    pub fn with_path(path: Option<&str>) -> Result<Self> {
        if let Some(path) = path {
            if !PathBuf::from(path).exists() {
                bail!("{} does not exist", path)
            }
        }
        let res = Figment::new()
            .merge(Toml::file(path.unwrap_or("./Caster.toml")))
            .merge(Env::prefixed("CASTER_").ignore(&["LOG"]))
            .extract()?;
        Ok(res)
    }

    pub fn new() -> Result<Self> {
        Self::with_path(None)
    }
}

fn default_log_level() -> String {
    "INFO".to_owned()
}

fn default_db_path() -> String {
    "/tmp/caster/sled".to_owned()
}

fn default_channel_size() -> usize {
    15
}

fn default_feed_interval() -> f64 {
    60.0
}

fn default_feed_ignore_days() -> u64 {
    30
}

fn default_telegram_content_max_length() -> usize {
    100
}
