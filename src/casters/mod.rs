mod_use::mod_use![feed, crates];

use std::{sync::Arc, time::SystemTime};

use color_eyre::{eyre::Context, Result};
use futures::future::join_all;
use log::error;
use once_cell::sync::OnceCell;
use sled::Db;
use tokio::sync::broadcast;

use crate::{Config, Event};

pub type TX = broadcast::Sender<Event>;
pub type RX = broadcast::Receiver<Event>;

static DB: OnceCell<Db> = OnceCell::new();

pub fn get_db<'a>() -> &'a Db {
    DB.get().expect("DB not initialized")
}

pub async fn run_casters(tx: TX, config: Arc<Config>) -> Result<TX> {
    let db = sled::open(&config.db_path).wrap_err("Failed to open db")?;
    drop(DB.set(db));

    let start = SystemTime::now();

    let mut handles = vec![];

    if let Some(ref rss) = config.caster_feed {
        handles.push(feed::run_feed(tx.clone(), rss.clone()))
    }

    if let Some(ref crates) = config.caster_crates {
        handles.push(crates::run_crates(tx.clone(), crates.clone()))
    }

    join_all(handles).await.into_iter().for_each(|res| {
        if let Err(e) = res {
            error!("{}", e)
        }
    });

    if start.elapsed()?.as_millis() < 500 {
        log::warn!("Caster shutting down too quickly, did you config both casters right?")
    }

    Ok(tx)
}
