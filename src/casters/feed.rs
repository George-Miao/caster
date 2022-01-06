use std::time::Duration;

use color_eyre::{
    eyre::{eyre, Context},
    Result,
};
use feed_rs::model::Feed;
use futures::{stream::FuturesUnordered, StreamExt};
use log::{debug, info, warn};
use tokio::task::JoinHandle;

use crate::{get_client, get_db, get_hash, ts_to_systemtime, Event, FeedConfig, TX};

const SECONDS_PER_DAY: u64 = 60 * 60 * 24;

pub fn run_feed(tx: TX, config: FeedConfig) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut timer = tokio::time::interval(Duration::from_secs_f64(config.interval));
        let db = get_db();
        let count = db.scan_prefix("FEED-").count();

        info!("Found {} feeds cached in DB", count);

        loop {
            timer.tick().await;
            let mut fut = config
                .urls
                .iter()
                .map(|url| fetch_one(url))
                .collect::<FuturesUnordered<_>>();

            while let Some(e) = fut.next().await {
                match e {
                    Err(e) => {
                        warn!("{}", e)
                    }
                    Ok((feed_id, feed)) => {
                        for entry in feed.entries.into_iter() {
                            let entry_id = format!("FEED-{}-{}", &feed_id, get_hash(entry.id));

                            let timestamp = entry
                                .published
                                .or(entry.updated)
                                .map(|time| time.timestamp())
                                .unwrap_or_default();

                            debug!("Entry fetched: {}, published at {}", entry_id, timestamp);

                            let data = timestamp.to_be_bytes();

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
                            } else {
                                // Entry does not exist, insert entry
                                if let Err(e) = db.insert(&entry_id, &data) {
                                    warn!("Failed to insert data to db: {}", e)
                                };

                                if let Ok(true) = ts_to_systemtime(timestamp as u64)
                                    .elapsed()
                                    .map(|x| x.as_secs() > SECONDS_PER_DAY * config.ignore_days)
                                {
                                    // Newly seen entry that is old, ignoring
                                    info!("Found old entry, ignored");
                                    return;
                                }
                            }

                            let title = entry.title.map(|title| title.content);
                            let link = entry.links.into_iter().next().map(|x| x.href);
                            let content = entry
                                .summary
                                .map(|content| content.content)
                                .or_else(|| entry.content.and_then(|x| x.body));

                            // Emit event
                            tx.send(Event::Feed {
                                time: timestamp,
                                entry_id,
                                content,
                                title,
                                link,
                            })
                            .expect("All consumers stopped");
                        }
                    }
                }

                if let Err(e) = db.flush_async().await {
                    warn!("Error flushing content to db: {}", e)
                }
            }
        }
    })
}

async fn fetch_one(url: &str) -> Result<(String, Feed)> {
    let res = get_client()
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
