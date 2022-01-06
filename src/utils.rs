use std::{
    collections::hash_map,
    hash::{Hash, Hasher},
    thread::sleep,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use color_eyre::{config::HookBuilder, eyre::Context, Result};
use humantime::Rfc3339Timestamp;
use once_cell::sync::Lazy;
use pretty_env_logger::formatted_timed_builder;
use reqwest::Client;

use crate::Config;

static CLIENT: Lazy<Client> = Lazy::new(Client::new);

pub fn init(config_path: Option<&str>) -> Result<Config> {
    HookBuilder::default()
        .display_env_section(false)
        .install()?;

    let config = Config::with_path(config_path)?;

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

pub fn ts_to_humantime(ts: u64) -> Rfc3339Timestamp {
    humantime::format_rfc3339(UNIX_EPOCH + Duration::from_secs(ts))
}

pub fn get_client<'a>() -> &'a Client {
    &CLIENT
}

pub struct Interval {
    interval: Duration,
    deadline: Option<Instant>,
}

impl Interval {
    pub fn every(interval: Duration) -> Self {
        Self {
            interval,
            deadline: None,
        }
    }

    pub fn next_tick(&mut self) -> Duration {
        let now = Instant::now();
        if self.deadline.is_none() {
            self.deadline = Some(now + self.interval);
            return Duration::from_micros(0);
        }
        let deadline = self.deadline.unwrap();
        if now > deadline {
            let mut point = deadline;
            loop {
                point += self.interval;
                if point > now {
                    break point - now;
                }
            }
        } else {
            deadline - now
        }
    }

    pub fn tick(&mut self) {
        sleep(self.next_tick())
    }
}
