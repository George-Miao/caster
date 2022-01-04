//!feedCasters
//! Structs for generating events from various source

mod_use::mod_use!(feed);

use color_eyre::{eyre::Context, Result};
use once_cell::sync::OnceCell;
use sled::Db;
use tokio::sync::broadcast::{self, Receiver, Sender};

use crate::{Config, Event};

pub type TX = Sender<Event>;
pub type RX = Receiver<Event>;

static DB: OnceCell<Db> = OnceCell::new();

pub fn get_db<'a>() -> &'a Db {
    DB.get().expect("DB not initialized")
}

pub fn run_casters(config: Config) -> Result<TX> {
    let db = sled::open(config.db_path).wrap_err("Failed to open db")?;
    drop(DB.set(db));
    let (tx, _) = broadcast::channel(config.channel_size);
    let mut feed_handle = None;
    if let Some(ref rss) = config.feed {
        feed_handle.replace(start_feed(tx.clone(), rss.clone()));
    }
    Ok(tx)
}
