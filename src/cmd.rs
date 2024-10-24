use std::fs;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::config::get_data_path;

#[derive(Serialize, Deserialize)]
pub struct SavedCommand {
    pub id: u32,
    pub command: String,

    #[serde(default = "default_description")]
    pub description: String,
}

#[derive(Serialize, Deserialize)]
pub struct ShelfData {
    commands: Vec<SavedCommand>,
}

fn default_description() -> String {
    "No description.".to_string()
}

fn get_next_id(commands: &Vec<SavedCommand>) -> u32 {
    commands.iter().map(|cmd| cmd.id).max().unwrap_or(0) + 1
}

pub fn save_command(command: String, description: Option<String>) -> Result<()> {
    // Get file
    let path = get_data_path(); // Path of the cmds.toml
    let mut shelf_data = if path.exists() {
        let content = fs::read_to_string(&path)?;
        toml::from_str(&content).context("Could not get toml data from string!")?
    } else {
        ShelfData { commands: vec![] }
    };
    // let mut shelf_data: ShelfData =
    //     toml::from_str(&content).context("Could not get toml data from string!")?;

    shelf_data.commands.push(SavedCommand {
        id: get_next_id(&shelf_data.commands),
        command,
        description: match description {
            Some(desc) => desc,
            None => default_description(),
        },
    });

    // Serialize data (save the command)

    let toml_string =
        toml::to_string(&shelf_data).context("Could not serialize data toml to string!")?;
    fs::write(&path, toml_string).context("Could not write command to data file!")?;

    Ok(())
}
