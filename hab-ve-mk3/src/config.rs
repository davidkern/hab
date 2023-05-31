use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::Path;

#[derive(Deserialize)]
pub struct Config {
    pub mk3_path: String,
    pub influxdb_url: String,
    pub influxdb_org: String,
    pub influxdb_token: String,
}

impl Config {
    /// Try to load config from current directory, or from the /etc/hab directory
    pub fn load() -> Result<Config> {
        Self::load_env()
            .or_else(|_| {
                let mut local_config_path = std::env::current_dir()?;
                local_config_path.push("hab-ve-mk3.toml");
        
                let content = Self::load_file(&local_config_path)
                    .or_else(|_| Config::load_file(Path::new("/etc/hab/hab-ve-mk3.toml")))
                    .with_context(|| format!("Failed to load config from {:?} or /etc/hab/hab-ve-mk3.toml", &local_config_path))?;
         
                toml::from_str(&content)
                    .with_context(|| "Failed to parse config file")        
            })
    }

    fn load_file(path: &Path) -> Result<String> {
        std::fs::read_to_string(path)
            .with_context(|| format!("Unable to read file {:?}", path))
    }

    fn load_env() -> Result<Config> {
        let vars: Vec<(String, String)> = std::env::vars().collect();
        println!("Environment:\n{:?}", vars);

        Ok(Config {
            mk3_path: std::env::var("MK3_PATH")?,
            influxdb_url: std::env::var("INFLUXDB_URL")?,
            influxdb_org: std::env::var("INFLUXDB_ORG")?,
            influxdb_token: std::env::var("INFLUXDB_TOKEN")?,
        })
    }
}