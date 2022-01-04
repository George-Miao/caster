use std::{env, sync::Arc};

use color_eyre::{eyre::Context, Result};
use futures::future::join;
use tokio::sync::broadcast;

mod_use::mod_use!(utils, config, event, casters, consumers,);

#[cfg(test)]
mod test;

#[tokio::main]
async fn main() -> Result<()> {
    run().await
}

async fn run() -> Result<()> {
    let config_dir = env::args().nth(1);
    let config = Arc::new(init(config_dir.as_deref())?);

    let (tx, _) = broadcast::channel(config.channel_size);

    match join(
        run_casters(tx.clone(), config.clone()),
        run_consumer(tx, config),
    )
    .await
    {
        (Err(e1), Err(e2)) => Err(e1
            .wrap_err("Caster error")
            .wrap_err(e2)
            .wrap_err("Consumer error")),
        (_, Err(e)) => Err(e).wrap_err("Consumer error"),
        (Err(e), _) => Err(e).wrap_err("Caster error"),
        _ => Ok(()),
    }
}
