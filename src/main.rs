use std::env;

use color_eyre::Result;

mod_use::mod_use!(utils, config, event, casters);

#[tokio::main]
async fn main() -> Result<()> {
    run().await
}

async fn run() -> Result<()> {
    let config_dir = env::args().nth(1);

    let config = init(config_dir.as_deref())?;

    let rx = run_casters(config)?;

    let mut tx = rx.subscribe();

    while let Ok(event) = tx.recv().await {
        log::info!("{}", event)
    }

    Ok(())
}
