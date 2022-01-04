mod_use::mod_use![feed];

use std::sync::Arc;

use color_eyre::{eyre::Context, Result};
use futures::future::join_all;
use log::error;
use once_cell::sync::OnceCell;
use sled::Db;
use tokio::sync::broadcast::{Receiver, Sender};

use crate::{Config, Event};

pub type TX = Sender<Event>;
pub type RX = Receiver<Event>;

static DB: OnceCell<Db> = OnceCell::new();

pub fn get_db<'a>() -> &'a Db {
    DB.get().expect("DB not initialized")
}

pub async fn run_casters(tx: TX, config: Arc<Config>) -> Result<TX> {
    let db = sled::open(&config.db_path).wrap_err("Failed to open db")?;
    drop(DB.set(db));

    let mut handles = vec![];
    if let Some(ref rss) = config.caster_feed {
        handles.push(feed::run_feed(tx.clone(), rss.clone()));
    }
    join_all(handles).await.into_iter().for_each(|res| {
        if let Err(e) = res {
            error!("{}", e)
        }
    });

    Ok(tx)
}
