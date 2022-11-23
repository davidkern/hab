use anyhow::Result;
use tokio::runtime::Runtime;

fn main() -> Result<()> {
    let rt = Runtime::new()?;

    rt.block_on(async {
        pretty_env_logger::init();

        log::debug!("exiting");
        Ok(())
    })
}