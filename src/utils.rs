use std::{
    collections::hash_map,
    hash::{Hash, Hasher},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use color_eyre::{eyre::Context, Result};
use pretty_env_logger::formatted_timed_builder;

use crate::Config;

pub fn init(config_path: Option<&str>) -> Result<Config> {
    let config = Config::with_path(config_path)?;

    color_eyre::install()?;

    let mut builder = formatted_timed_builder();
    builder.parse_filters(&config.log_level);
    builder.try_init().wrap_err("Failed to init logger")?;

    log::debug!("Config: {:#?}", config);

    Ok(config)
}

pub fn get_hash(val: impl Hash) -> String {
    let mut hasher = hash_map::DefaultHasher::new();
    val.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

pub fn ts_to_systemtime(ts: u64) -> SystemTime {
    UNIX_EPOCH + Duration::from_secs(ts)
}
