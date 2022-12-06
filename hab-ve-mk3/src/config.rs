use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::Path;

#[derive(Deserialize)]
pub struct Config {
    pub mk3_port: String,
    pub influxdb_url: String,
    pub influxdb_org: String,
    pub influxdb_key: String,
}

impl Config {
    /// Try to load config from current directory, or from the /etc/hab directory
    pub fn load() -> Result<Config> {
        let mut local_config_path = std::env::current_dir()?;
        local_config_path.push("hab-ve-mk3.toml");

        let content = Self::load_file(&local_config_path)
            .or_else(|_| Config::load_file(Path::new("/etc/hab/hab-ve-mk3.toml")))
            .with_context(|| format!("Failed to load config from {:?} or /etc/hab/hab-ve-mk3.toml", &local_config_path))?;
 
        toml::from_str(&content)
            .with_context(|| "Failed to parse config file")
    }

    fn load_file(path: &Path) -> Result<String> {
        std::fs::read_to_string(path)
            .with_context(|| format!("Unable to read file {:?}", path))
    }
}