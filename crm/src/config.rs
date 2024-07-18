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
    pub sender_email: String,
    pub metadata: String,
    pub user_stats: String,
    pub notification: String,
}

impl AppConfig {
    pub fn try_load() -> Result<Self> {
        let reader = File::open("./crm.yml")
            .or_else(|_| File::open("/etc/config/crm.yml"))
            .or_else(|_| File::open(env::var("CRM_CONFIG").expect("Config file not found")))?;
        Self::load_from_reader(reader)
    }

    pub fn load_from_reader<R: std::io::Read>(reader: R) -> Result<Self> {
        Ok(serde_yaml::from_reader(reader)?)
    }
}
