mod mk3;

use anyhow::Result;
use tokio::runtime::Runtime;

fn main() -> Result<()> {
    let rt = Runtime::new()?;

    rt.block_on(async {
        pretty_env_logger::init();

        mk3::run("/tmp/mk3").await?;

        log::debug!("exiting");
        Ok(())
    })
}