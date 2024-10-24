use std::{fs, path::PathBuf};

use anyhow::{Context, Result};
use dirs::{config_dir, data_dir};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub storage_path: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            storage_path: get_data_path().display().to_string(),
        }
    }
}

pub fn get_config_dir() -> PathBuf {
    config_dir().unwrap_or_default().join("shelf")
}

pub fn get_config_path(config_dir: &PathBuf) -> PathBuf {
    config_dir.join("config.toml")
}

pub fn get_data_path() -> PathBuf {
    data_dir().unwrap_or_default().join("shelf/cmds.toml")
}

pub fn load_config(config_dir: &PathBuf, config_path: &PathBuf) -> Result<Config> {
    // Create directories if they don't exist
    fs::create_dir_all(config_dir).context("Could not create `shelf` directory")?;

    if !config_path.exists() {
        let default_config = Config::default();
        let toml_string =
            toml::to_string(&default_config).context("Could not serialize toml to string")?;
        fs::write(config_path, toml_string).context("Could not write default config!")?;
        return Ok(default_config);
    }

    let content =
        fs::read_to_string(config_path).context("Could not read default path to string!")?;
    let config: Config = toml::from_str(&content).context("Could not get toml from string")?;

    Ok(config)
}
