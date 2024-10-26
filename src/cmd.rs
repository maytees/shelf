use std::{fs, process::Command};

use anyhow::{Context, Error, Result};
use copypasta::{ClipboardContext, ClipboardProvider};
use serde::{Deserialize, Serialize};
use shellexpand;

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

fn get_shelf_data() -> Result<ShelfData, Error> {
    let path = get_data_path(); // Path of the cmds.toml
    if path.exists() {
        let content = fs::read_to_string(&path)?;
        return Ok(toml::from_str(&content).context("Could not get toml data from string!")?);
    }

    Ok(ShelfData { commands: vec![] })
}

pub fn save_command(command: String, description: Option<String>) -> Result<()> {
    // Get file
    let mut shelf_data = get_shelf_data().context("Could not fetch shelf data")?;

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
    fs::write(&get_data_path(), toml_string).context("Could not write command to data file!")?;

    println!(
        "{} {} {}",
        "Shelved command: ".green(),
        command.cyan().bold(),
        "succesfully".green()
    );

    Ok(())
}

pub fn list_commands(verbose: &bool, reverse: &bool) -> Result<()> {
    let mut shelf_data = get_shelf_data().context("Could not fetch shelf data")?;

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

pub fn copy_command(id: &u32) -> Result<()> {
    let shelf_data = get_shelf_data().context("Could not fetch shelf data")?;
    let mut ctx = ClipboardContext::new().unwrap();

    if let Some(cmd) = shelf_data.commands.iter().find(|cmd| cmd.id == *id) {
        ctx.set_contents(cmd.command.clone()).unwrap();

        println!(
            "{} {} {}",
            "Saved".green(),
            cmd.command.cyan().bold(),
            "to your clipboard.".green()
        );

        return Ok(());
    }

    eprintln!(
        "{}{}",
        "Could not find saved command with id: ".red(),
        id.to_string().yellow().bold()
    );

    std::process::exit(1)
}

pub fn run_command(id: &u32) -> Result<()> {
    let shelf_data = get_shelf_data().context("Could not fetch shelf data")?;
    if let Some(cmd) = shelf_data.commands.iter().find(|cmd| cmd.id == *id) {
        // First expand any environment variables in the command
        let expanded_command = shellexpand::full(&cmd.command)
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
    eprintln!(
        "{}{}",
        "Could not find saved command with id: ".red(),
        id.to_string().yellow().bold()
    );
    std::process::exit(1)
}
