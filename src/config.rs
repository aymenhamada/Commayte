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
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("commayte");

    // Create config directory if it doesn't exist
    if !config_dir.exists() {
        let _ = fs::create_dir_all(&config_dir);
    }

    config_dir.join("config.toml")
}

pub fn load_config() -> Config {
    let config_path = get_config_path();

    if let Ok(config_content) = fs::read_to_string(&config_path) {
        if let Ok(config) = toml::from_str(&config_content) {
            return config;
        }
    }

    // Return default config if file doesn't exist or is invalid
    Config::default()
}
