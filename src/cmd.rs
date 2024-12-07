use anyhow::{Context, Error, Result};
use copypasta::{ClipboardContext, ClipboardProvider};
use serde::{Deserialize, Serialize};
use shellexpand;
use std::{fmt::Display, fs, process::Command};

use crate::{config::get_data_path, fuzzy::FuzzyPicker};
extern crate colored; // not needed in Rust 2018+
use colored::*;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SavedCommand {
    pub id: u32,
    pub command: String,

    #[serde(default = "default_description")]
    pub description: String,

    pub tags: Option<Vec<String>>,
}

fn default_description() -> String {
    "No description.".to_string()
}

// Used to display da' fuzz
impl Display for SavedCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut tagout = String::new();

        if let Some(tags) = &self.tags {
            tagout.push_str(
                format!(" {} {}", "-- Tags:".red().bold(), tags.join(", ").red()).as_str(),
            );
        }

        write!(
            f,
            "{} {} {} {} {}{}",
            self.id.to_string().yellow(),
            "-".to_string().yellow(),
            self.command.red().bold(),
            "--".to_string().yellow(),
            self.description.yellow(),
            tagout
        )
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ShelfData {
    commands: Vec<SavedCommand>,
}

fn get_next_id(commands: &Vec<SavedCommand>) -> u32 {
    commands.iter().map(|cmd| cmd.id).max().unwrap_or(0) + 1
}

fn get_shelf_data() -> Result<ShelfData, Error> {
    let path = get_data_path(); // Path of the cmds.toml
    if path.exists() {
        let content = fs::read_to_string(&path)?;
        return Ok(toml::from_str(&content).context("Could not get toml data from string!")?);
    }

    Ok(ShelfData { commands: vec![] })
}

pub fn save_command(
    command: String,
    description: Option<String>,
    tags: Option<Vec<String>>,
) -> Result<()> {
    // Get file
    let mut shelf_data = get_shelf_data().context("Could not fetch shelf data")?;

    shelf_data.commands.push(SavedCommand {
        id: get_next_id(&shelf_data.commands),
        command: command.clone(),
        description: match description {
            Some(desc) => desc,
            None => default_description(),
        },
        tags,
    });

    // Serialize data (save the command)
    let toml_string =
        toml::to_string(&shelf_data).context("Could not serialize data toml to string!")?;
    fs::write(&get_data_path(), toml_string).context("Could not write command to data file!")?;

    println!(
        "{} {} {}",
        "Shelved command:".green(),
        command.cyan().bold(),
        "succesfully".green()
    );

    Ok(())
}

pub fn list_commands(verbose: &bool, reverse: &bool, limit: &Option<u32>) -> Result<()> {
    let mut shelf_data = get_shelf_data().context("Could not fetch shelf data")?;

    if *reverse {
        shelf_data.commands.reverse();
    }

    if let Some(limit) = limit {
        shelf_data.commands.truncate(*limit as usize);
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
            cmd.command.bright_cyan().bold(),
        );

        if *verbose {
            output.push_str(
                format!(
                    "\n  {} {}",
                    "-- Desc: ".yellow().bold(),
                    cmd.description.yellow()
                )
                .as_str(),
            );

            if let Some(tags) = &cmd.tags {
                output.push_str(
                    format!(
                        "\n  {} {}",
                        "-- Tags: ".yellow().bold(),
                        tags.join(", ").yellow()
                    )
                    .as_str(),
                );
            }
        }

        println!("{}", output);
    });
    Ok(())
}

fn save_to_clipboard(cmd: &SavedCommand) -> Result<()> {
    let mut ctx = ClipboardContext::new().unwrap();
    ctx.set_contents(cmd.command.clone()).unwrap();

    println!(
        "{} {} {}",
        "Saved".green(),
        cmd.command.cyan().bold(),
        "to your clipboard.".green()
    );

    Ok(())
}

pub fn copy_command(id: &u32) -> Result<()> {
    let shelf_data = get_shelf_data().context("Could not fetch shelf data")?;

    if let Some(cmd) = shelf_data.commands.iter().find(|cmd| cmd.id == *id) {
        return save_to_clipboard(cmd);
    }

    eprintln!(
        "{}{}",
        "Could not find saved command with id: ".red(),
        id.to_string().yellow().bold()
    );

    std::process::exit(1)
}

fn exec_command(command: SavedCommand) -> Result<()> {
    // First expand any environment variables in the command
    let expanded_command = shellexpand::full(&command.command)
        .map_err(|e| anyhow::anyhow!("Failed to expand environment variables: {}", e))?;

    // Split the expanded command string into parts
    let args: Vec<String> = expanded_command
        .split_whitespace()
        .map(|s| s.to_string())
        .collect();

    if args.is_empty() {
        return Err(anyhow::anyhow!("Empty command after expansion"));
    }

    let command = &args[0];
    let params = &args[1..];

    // Execute the command
    match Command::new(command).args(params).status() {
        Ok(status) => {
            if !status.success() {
                eprintln!("Command failed with status: {}", status);
            }
        }
        Err(e) => eprintln!("Failed to execute command: {}, {:?}", e, args),
    }
    return Ok(());
}

pub fn run_command(id: &u32) -> Result<()> {
    let shelf_data = get_shelf_data().context("Could not fetch shelf data")?;
    if let Some(cmd) = shelf_data.commands.iter().find(|cmd| cmd.id == *id) {
        return exec_command(cmd.clone());
    }
    eprintln!(
        "{}{}",
        "Could not find saved command with id: ".red(),
        id.to_string().yellow().bold()
    );
    std::process::exit(1)
}

pub fn fuzzy_search(copy: &bool) -> Result<()> {
    let shelf_data = get_shelf_data().context("Could not fetch shelf data")?;

    let mut picker = FuzzyPicker::new(&shelf_data.commands);

    if let Ok(Some(selected)) = picker.pick() {
        if *copy {
            return save_to_clipboard(&selected);
        }

        return exec_command(selected);
    } else {
        println!("{}", "No saved command selected...".red().bold());
    }

    Ok(())
}
