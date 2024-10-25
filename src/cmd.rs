use std::fs;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::config::get_data_path;
extern crate colored; // not needed in Rust 2018+
use colored::*;

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

    shelf_data.commands.push(SavedCommand {
        id: get_next_id(&shelf_data.commands),
        command: command.clone(),
        description: match description {
            Some(desc) => desc,
            None => default_description(),
        },
    });

    // Serialize data (save the command)
    let toml_string =
        toml::to_string(&shelf_data).context("Could not serialize data toml to string!")?;
    fs::write(&path, toml_string).context("Could not write command to data file!")?;

    println!(
        "{} {} {}",
        "Shelved command:".green(),
        command.cyan().bold(),
        "succesfully".green()
    );

    Ok(())
}

pub fn list_commands(verbose: &bool, reverse: &bool) -> Result<()> {
    let path = get_data_path();
    let mut shelf_data = if path.exists() {
        let content = fs::read_to_string(&path)?;
        toml::from_str(&content).context("Could not get toml data from string!")?
    } else {
        ShelfData { commands: vec![] }
    };

    if *reverse {
        shelf_data.commands.reverse();
    }

    if shelf_data.commands.is_empty() {
        println!("{}", "You have no saved commands!".red());
        return Ok(());
    }

    shelf_data.commands.iter().for_each(|cmd| {
        let mut output = format!(
            "{} {} {}",
            cmd.id.to_string().yellow().bold(),
            "-".bright_yellow().bold(),
            cmd.command.bright_magenta().bold(),
        );

        if *verbose {
            output
                .push_str(format!(" {} {}", "--".bright_yellow().bold(), cmd.description).as_str());
        }

        println!("{}", output);
    });
    Ok(())
}
