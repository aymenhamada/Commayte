use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub model: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            model: "mistral".to_string(),
        }
    }
}

fn get_config_path() -> PathBuf {
    // Always use ~/.config/commayte to match the install script
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".config")
        .join("commayte")
        .join("config.toml")
}

pub fn load_config() -> Config {
    let config_path = get_config_path();

    match fs::read_to_string(&config_path) {
        Ok(config_content) => match toml::from_str(&config_content) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("Warning: Failed to parse config file: {e}");
                eprintln!("Using default configuration");
                Config::default()
            }
        },
        Err(e) => {
            eprintln!("Warning: Could not read config file at {config_path:?}: {e}",);
            eprintln!("Using default configuration");
            Config::default()
        }
    }
}
