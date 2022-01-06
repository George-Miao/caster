use std::{sync::Arc, time::SystemTime};

use color_eyre::Result;
use futures::future::join_all;
use log::error;
use tokio::task::JoinHandle;

use crate::{Config, TX};

mod telegram;

pub async fn run_consumer(tx: TX, config: Arc<Config>) -> Result<()> {
    let start = SystemTime::now();

    let mut handles: Vec<JoinHandle<()>> = vec![];

    if let Some(ref config) = config.consumer_telegram {
        handles.push(telegram::run_telegram(tx.subscribe(), config.clone()));
    }

    join_all(handles).await.into_iter().for_each(|res| {
        if let Err(e) = res {
            error!("{}", e)
        }
    });

    if start.elapsed()?.as_millis() < 500 {
        log::warn!("Consumers shutting down too quickly, did you config both consumers right?")
    }
    Ok(())
}
