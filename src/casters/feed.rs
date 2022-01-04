use std::time::Duration;

use color_eyre::{
    eyre::{eyre, Context},
    Result,
};
use feed_rs::model::Feed;
use futures::{stream::FuturesUnordered, StreamExt};
use log::{debug, info, warn};
use reqwest::Client;
use tokio::task::JoinHandle;

use crate::{get_db, get_hash, Event, FeedConfig, TX};

pub fn start_feed(tx: TX, config: FeedConfig) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut timer = tokio::time::interval(Duration::from_secs_f64(config.interval));
        let db = get_db();
        let count = db.scan_prefix("FEED-").count();

        info!("Found {} feeds cached in DB", count);

        let client = reqwest::Client::new();
        loop {
            timer.tick().await;
            let mut fut = config
                .urls
                .iter()
                .map(|url| fetch_one(client.clone(), url))
                .collect::<FuturesUnordered<_>>();

            while let Some(e) = fut.next().await {
                match e {
                    Err(e) => {
                        warn!("{}", e)
                    }
                    Ok((feed_id, feed)) => feed.entries.into_iter().for_each(|entry| {
                        let entry_id = format!("FEED-{}-{}", &feed_id, get_hash(entry.id));

                        let timestamp = entry
                            .published
                            .or(entry.updated)
                            .map(|time| time.timestamp())
                            .unwrap_or_default();

                        debug!("Entry fetched: {}, updated at {}", entry_id, timestamp);

                        let data: [u8; 8] = timestamp.to_be_bytes();

                        // Entry already exists
                        if let Ok(Some(ref v)) = db.get(&entry_id) {
                            // Entry not updated, early return
                            if v == &data {
                                return;
                            }
                            // Update entry
                            if let Err(e) = db.insert(&entry_id, &data) {
                                warn!("Failed to insert data to db: {}", e)
                            };
                            tx.send(Event::Feed {
                                entry_id,
                                time: timestamp,
                                payload: entry
                                    .content
                                    .and_then(|content| content.body)
                                    .unwrap_or_default(),
                            })
                            .expect("All subscribers dropped");
                        } else {
                            // Entry does not exist, insert entry
                            if let Err(e) = db.insert(&entry_id, &data) {
                                warn!("Failed to insert data to db: {}", e)
                            };
                            tx.send(Event::Feed {
                                entry_id,
                                time: timestamp,
                                payload: entry
                                    .content
                                    .and_then(|content| content.body)
                                    .unwrap_or_default(),
                            })
                            .expect("All subscribers dropped");
                        }
                    }),
                }

                if let Err(e) = db.flush_async().await {
                    warn!("Error flushing content to db: {}", e)
                }
            }
        }
    })
}

async fn fetch_one(client: Client, url: &str) -> Result<(String, Feed)> {
    let res = client
        .get(url)
        .send()
        .await
        .wrap_err_with(|| format!("Request failed: {}", url))?;
    let status = res.status();
    if !status.is_success() {
        let text = res.text().await.wrap_err("Decode failed")?;
        Err(eyre!("{}", text).wrap_err(format!(
            "Unsuccessful response from server (Code: {})",
            status
        )))
    } else {
        let bytes = res.bytes().await?;
        let feed = feed_rs::parser::parse(bytes.as_ref()).wrap_err("Failed to parse feed")?;
        Ok((get_hash(url), feed))
    }
}
