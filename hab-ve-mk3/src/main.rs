mod config;
mod decoder;
mod mk3;

use anyhow::Result;
use tokio::runtime::Runtime;

fn main() -> Result<()> {
    let config = config::Config::load()?;

    let rt = Runtime::new()?;
    rt.block_on(async move {
        pretty_env_logger::init();

        mk3::run(&config).await?;

        log::debug!("exiting");
        Ok(())
    })
}