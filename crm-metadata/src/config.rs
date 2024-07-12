use std::{env, fs::File};

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct AppConfig {
    pub server: ServerConfig,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ServerConfig {
    pub port: u16,
}

impl AppConfig {
    pub fn try_load() -> Result<Self> {
        let reader = File::open("./metadata.yml")
            .or_else(|_| File::open("/etc/config/metadata.yml"))
            .or_else(|_| File::open(env::var("METADATA_CONFIG").expect("Config file not found")))?;
        Self::load_from_reader(reader)
    }

    pub fn load_from_reader<R: std::io::Read>(reader: R) -> Result<Self> {
        Ok(serde_yaml::from_reader(reader)?)
    }
}
