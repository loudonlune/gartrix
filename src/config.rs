
use std::{fs, env};

use lazy_static::lazy_static;
use serde_derive::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Default, Clone)]
pub struct Config {
    base_url: String,
    database_url: String,
}

static CONFIG_PATH: &str = "config.json";

lazy_static! {
    pub static ref GLOBAL_CONFIG: Config = load_config();
}

pub fn load_config() -> Config {
    let mut config: Config = if let Ok(cfg) = fs::read_to_string(CONFIG_PATH) {
            serde_json::from_str(cfg.as_str()).expect("The configuration file is invalid.")
        } else {
            Config::default()
        };

    config.base_url = env::var("GARTRIX_BASE_URL").unwrap_or(config.base_url);
    config.database_url = env::var("GARTRIX_DATABASE_URL").unwrap_or(config.database_url);

    config
}

pub fn write_config() {
    let cfg: Config = GLOBAL_CONFIG.clone();

    fs::write(
        CONFIG_PATH, 
        serde_json::to_string(&cfg).unwrap()
    )
    .expect("Error: Failed to write a new configuration file.");
}