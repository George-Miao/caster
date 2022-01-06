use std::time::Duration;

use log::{debug, info, warn};
use tokio::task::{spawn_blocking, JoinHandle};

use crate::{get_db, CratesConfig, Event, Interval, TX};

pub fn run_crates(tx: TX, config: CratesConfig) -> JoinHandle<()> {
    spawn_blocking(move || {
        let db = get_db();
        let mut index = crates_index::Index::new_cargo_default().unwrap();
        let mut interval = Interval::every(Duration::from_secs_f64(config.interval));

        loop {
            info!("Fetching crates.io index");
            interval.tick();
            if let Err(e) = index.update() {
                warn!("Failed to update crates.io index: {}", e)
            }

            for crate_name in config.crates.iter() {
                let crate_item = if let Some(x) = index.crate_(crate_name) {
                    x
                } else {
                    warn!("Cannot find crate `{}`", crate_name);
                    continue;
                };
                let ver = crate_item.latest_version();
                let ver_id = format!("CRATE-{}", crate_name);
                let cksm = ver.checksum();

                debug!("Checksum: {}", hex::encode(cksm));

                if let Ok(Some(cksm_in_db)) = db.get(&ver_id) {
                    if cksm_in_db == cksm {
                        continue;
                    }
                }

                if let Err(e) = db.insert(ver_id, cksm) {
                    warn!("Failed to insert data to db: {e}")
                }

                tx.send(Event::CratesIo {
                    name: crate_name.to_owned(),
                    vers: ver.version().to_owned(),
                    links: ver.links().map(Into::into),
                    yanked: ver.is_yanked(),
                })
                .expect("All subscribers dropped");
            }
        }
    })
}
