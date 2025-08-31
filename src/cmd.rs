use anyhow::{Context, Error, Result};
use copypasta::{ClipboardContext, ClipboardProvider};
use regex::Regex;
use serde::{Deserialize, Serialize};
use shellexpand;
use std::{
    fmt::Display,
    fs,
    io::{self, Write},
    process::Command,
};

use crate::{config::{ensure_data_dir_exists, get_data_path}, fuzzy::FuzzyPicker};
extern crate colored; // not needed in Rust 2018+
use colored::*;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SavedCommand {
    pub id: u32,
    pub command: String,

    #[serde(default = "default_description")]
    pub description: String,

    pub tags: Option<Vec<String>>,

    #[serde(default = "default_is_template")]
    pub is_template: bool,
}

fn default_is_template() -> bool {
    false
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

fn extract_parameters(command: &str) -> Vec<String> {
    let re = Regex::new(r"\{\{(\w+)\}\}").unwrap();
    let mut params = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for cap in re.captures_iter(command) {
        let start_pos = cap.get(0).unwrap().start();

        // Check if this match is escaped (preceded by backslash)
        if start_pos > 0 && command.chars().nth(start_pos - 1) == Some('\\') {
            continue; // Skip escaped templates
        }

        let param = cap[1].to_string();
        if seen.insert(param.clone()) {
            params.push(param);
        }
    }

    params
}

fn prompt_for_parameters(
    parameters: &[String],
) -> Result<std::collections::HashMap<String, String>> {
    let mut values = std::collections::HashMap::new();

    for param in parameters {
        print!("Enter {}: ", param.yellow().bold());
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let value = input.trim().to_string();

        // if value.is_empty() {
        //     return Err(anyhow::anyhow!("Parameter '{}' cannot be empty", param));
        // }

        values.insert(param.clone(), value);
    }

    Ok(values)
}

fn interpolate_command(
    command: &str,
    values: &std::collections::HashMap<String, String>,
) -> String {
    let mut result = command.to_string();

    for (param, value) in values {
        let pattern = format!("{{{{{}}}}}", param);
        result = result.replace(&pattern, value);
    }

    // Remove backslashes that were used to escape template syntax
    result = result.replace("\\{{", "{{");

    result
}

fn get_shelf_data() -> Result<ShelfData, Error> {
    let path = get_data_path(); // Path of the cmds.toml
    if path.exists() {
        let content = fs::read_to_string(&path)?;
        let mut shelf_data: ShelfData =
            toml::from_str(&content).context("Could not get toml data from string!")?;

        // Migrate old commands that don't have is_template field
        for command in &mut shelf_data.commands {
            if command.is_template == false && !extract_parameters(&command.command).is_empty() {
                command.is_template = true;
            }
        }

        return Ok(shelf_data);
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

    let parameters = extract_parameters(&command);
    let is_template = !parameters.is_empty();

    if is_template {
        println!(
            "{} {}",
            "Template detected with parameters:".yellow(),
            parameters.join(", ").cyan().bold()
        );
    }

    shelf_data.commands.push(SavedCommand {
        id: get_next_id(&shelf_data.commands),
        command: command.clone(),
        description: match description {
            Some(desc) => desc,
            None => default_description(),
        },
        tags,
        is_template,
    });

    // Ensure data directory exists before writing
    ensure_data_dir_exists().context("Could not create data directory")?;

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
            if cmd.description != "No description." {
                output.push_str(
                    format!(
                        "\n  {} {}",
                        "-- Desc: ".yellow().bold(),
                        cmd.description.yellow()
                    )
                    .as_str(),
                );
            }

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
    let final_command = if command.is_template {
        let parameters = extract_parameters(&command.command);
        if !parameters.is_empty() {
            println!(
                "{}",
                "This is a template command. Please provide values:".yellow()
            );
            let values = prompt_for_parameters(&parameters)?;
            interpolate_command(&command.command, &values)
        } else {
            command.command.clone()
        }
    } else {
        command.command.clone()
    };

    // First expand any environment variables in the command
    let expanded_command = shellexpand::full(&final_command)
        .map_err(|e| anyhow::anyhow!("Failed to expand environment variables: {}", e))?;

    // Split the expanded command string into parts
    let args: Vec<String> = expanded_command
        .split_whitespace()
        .map(|s| s.to_string())
        .collect();

    if args.is_empty() {
        return Err(anyhow::anyhow!("Empty command after expansion"));
    }

    let command_name = &args[0];
    let params = &args[1..];

    // Execute the command
    match Command::new(command_name).args(params).status() {
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

pub fn delete_command(id: &u32) -> Result<()> {
    let mut shelf_data = get_shelf_data().context("Could not fetch shelf data")?;

    let initial_len = shelf_data.commands.len();
    shelf_data.commands.retain(|cmd| cmd.id != *id);

    if shelf_data.commands.len() == initial_len {
        eprintln!(
            "{}{}",
            "Could not find saved command with id: ".red(),
            id.to_string().yellow().bold()
        );
        std::process::exit(1);
    }

    ensure_data_dir_exists().context("Could not create data directory")?;
    let toml_string =
        toml::to_string(&shelf_data).context("Could not serialize data toml to string!")?;
    fs::write(&get_data_path(), toml_string).context("Could not write updated data to file!")?;

    println!(
        "{} {} {}",
        "Deleted command with id:".green(),
        id.to_string().yellow().bold(),
        "successfully".green()
    );

    Ok(())
}

pub fn remove_tag(id: &u32, tag: &String) -> Result<()> {
    let mut shelf_data = get_shelf_data().context("Could not fetch shelf data")?;

    if let Some(cmd) = shelf_data.commands.iter_mut().find(|cmd| cmd.id == *id) {
        if let Some(tags) = &mut cmd.tags {
            let initial_len = tags.len();
            tags.retain(|t| t != tag);

            if tags.len() == initial_len {
                eprintln!(
                    "{}{} {} {}",
                    "Tag ".red(),
                    tag.yellow().bold(),
                    "not found in command with id:".red(),
                    id.to_string().yellow().bold()
                );
                std::process::exit(1);
            }

            if tags.is_empty() {
                cmd.tags = None;
            }
        } else {
            eprint!(
                "{} {}",
                "Command with id:".red(),
                id.to_string().yellow().bold(),
            );
            eprintln!("{}", " has no tags to remove.".red());
            std::process::exit(1);
        }

        let toml_string =
            toml::to_string(&shelf_data).context("Could not serialize data toml to string!")?;
        fs::write(&get_data_path(), toml_string)
            .context("Could not write updated data to file!")?;

        println!(
            "{} {} {} {} {}",
            "Removed tag".green(),
            tag.yellow().bold(),
            "from command with id:".green(),
            id.to_string().yellow().bold(),
            "successfully".green()
        );
    } else {
        eprintln!(
            "{}{}",
            "Could not find saved command with id: ".red(),
            id.to_string().yellow().bold()
        );
        std::process::exit(1);
    }

    Ok(())
}

pub fn add_tag(id: &u32, tag: &String) -> Result<()> {
    let mut shelf_data = get_shelf_data().context("Could not fetch shelf data")?;

    if let Some(cmd) = shelf_data.commands.iter_mut().find(|cmd| cmd.id == *id) {
        if let Some(tags) = &mut cmd.tags {
            if tags.contains(tag) {
                eprintln!(
                    "{}{} {} {}",
                    "Tag ".red(),
                    tag.yellow().bold(),
                    "already exists in command with id:".red(),
                    id.to_string().yellow().bold()
                );
                std::process::exit(1);
            }
            tags.push(tag.clone());
        } else {
            cmd.tags = Some(vec![tag.clone()]);
        }

        let toml_string =
            toml::to_string(&shelf_data).context("Could not serialize data toml to string!")?;
        fs::write(&get_data_path(), toml_string)
            .context("Could not write updated data to file!")?;

        println!(
            "{} {} {} {} {}",
            "Added tag".green(),
            tag.yellow().bold(),
            "to command with id:".green(),
            id.to_string().yellow().bold(),
            "successfully".green()
        );
    } else {
        eprintln!(
            "{}{}",
            "Could not find saved command with id: ".red(),
            id.to_string().yellow().bold()
        );
        std::process::exit(1);
    }

    Ok(())
}

pub fn edit_description(id: &u32, new_description: &String) -> Result<()> {
    let mut shelf_data = get_shelf_data().context("Could not fetch shelf data")?;

    if let Some(cmd) = shelf_data.commands.iter_mut().find(|cmd| cmd.id == *id) {
        let old_description = cmd.description.clone();
        cmd.description = new_description.clone();

        let toml_string =
            toml::to_string(&shelf_data).context("Could not serialize data toml to string!")?;
        fs::write(&get_data_path(), toml_string)
            .context("Could not write updated data to file!")?;

        println!(
            "{} {} {} {}",
            "Updated description for command with id:".green(),
            id.to_string().yellow().bold(),
            "successfully".green(),
            format!("({} -> {})", old_description, new_description).bright_black()
        );
    } else {
        eprintln!(
            "{}{}",
            "Could not find saved command with id: ".red(),
            id.to_string().yellow().bold()
        );
        std::process::exit(1);
    }

    Ok(())
}

pub fn edit_command_string(id: &u32, new_command: &String) -> Result<()> {
    let mut shelf_data = get_shelf_data().context("Could not fetch shelf data")?;

    if let Some(cmd) = shelf_data.commands.iter_mut().find(|cmd| cmd.id == *id) {
        let old_command = cmd.command.clone();
        cmd.command = new_command.clone();

        let toml_string =
            toml::to_string(&shelf_data).context("Could not serialize data toml to string!")?;
        fs::write(&get_data_path(), toml_string)
            .context("Could not write updated data to file!")?;

        println!(
            "{} {} {} {}",
            "Updated command with id:".green(),
            id.to_string().yellow().bold(),
            "successfully".green(),
            format!("({} -> {})", old_command, new_command).bright_black()
        );
    } else {
        eprintln!(
            "{}{}",
            "Could not find saved command with id: ".red(),
            id.to_string().yellow().bold()
        );
        std::process::exit(1);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::sync::Mutex;
    use tempfile::TempDir;

    static TEST_MUTEX: Mutex<()> = Mutex::new(());

    struct TestGuard {
        _temp_dir: TempDir,
        _lock: std::sync::MutexGuard<'static, ()>,
    }

    fn setup_test_env() -> TestGuard {
        let lock = TEST_MUTEX.lock().unwrap();
        let temp_dir = TempDir::new().unwrap();

        // Set environment variables to point to temp directory
        env::set_var("SHELF_DATA_DIR", temp_dir.path().to_str().unwrap());
        env::set_var("SHELF_CONFIG_DIR", temp_dir.path().to_str().unwrap());

        TestGuard {
            _temp_dir: temp_dir,
            _lock: lock,
        }
    }

    impl Drop for TestGuard {
        fn drop(&mut self) {
            env::remove_var("SHELF_DATA_DIR");
            env::remove_var("SHELF_CONFIG_DIR");
        }
    }

    #[test]
    fn test_save_and_list_command() {
        let _guard = setup_test_env();

        let result = save_command(
            "echo hello".to_string(),
            Some("Test command".to_string()),
            Some(vec!["test".to_string()]),
        );
        assert!(result.is_ok());

        let shelf_data = get_shelf_data().unwrap();
        assert_eq!(shelf_data.commands.len(), 1);
        assert_eq!(shelf_data.commands[0].command, "echo hello");
        assert_eq!(shelf_data.commands[0].description, "Test command");
        assert_eq!(shelf_data.commands[0].tags, Some(vec!["test".to_string()]));
    }

    #[test]
    fn test_template_detection() {
        let _guard = setup_test_env();

        save_command(
            "ssh {{user}}@{{host}}".to_string(),
            Some("SSH template".to_string()),
            None,
        )
        .unwrap();

        let shelf_data = get_shelf_data().unwrap();
        assert_eq!(shelf_data.commands.len(), 1);
        assert!(shelf_data.commands[0].is_template);

        let params = extract_parameters(&shelf_data.commands[0].command);
        assert_eq!(params, vec!["user", "host"]);
    }

    #[test]
    fn test_escaped_template() {
        let _guard = setup_test_env();

        save_command(
            "echo \\{{literal}}".to_string(),
            Some("Escaped template".to_string()),
            None,
        )
        .unwrap();

        let shelf_data = get_shelf_data().unwrap();
        assert_eq!(shelf_data.commands.len(), 1);
        assert!(!shelf_data.commands[0].is_template);

        let params = extract_parameters(&shelf_data.commands[0].command);
        assert!(params.is_empty());
    }

    #[test]
    fn test_add_and_remove_tag() {
        let _guard = setup_test_env();

        save_command(
            "echo test".to_string(),
            Some("Test".to_string()),
            Some(vec!["initial".to_string()]),
        )
        .unwrap();

        let shelf_data = get_shelf_data().unwrap();
        let id = shelf_data.commands[0].id;

        add_tag(&id, &"newtag".to_string()).unwrap();
        let shelf_data = get_shelf_data().unwrap();
        let tags = shelf_data.commands[0].tags.as_ref().unwrap();
        assert!(tags.contains(&"newtag".to_string()));
        assert!(tags.contains(&"initial".to_string()));

        remove_tag(&id, &"initial".to_string()).unwrap();
        let shelf_data = get_shelf_data().unwrap();
        let tags = shelf_data.commands[0].tags.as_ref().unwrap();
        assert!(tags.contains(&"newtag".to_string()));
        assert!(!tags.contains(&"initial".to_string()));
    }

    #[test]
    fn test_edit_description_and_command() {
        let _guard = setup_test_env();

        save_command("echo old".to_string(), Some("Old desc".to_string()), None).unwrap();

        let shelf_data = get_shelf_data().unwrap();
        let id = shelf_data.commands[0].id;

        edit_description(&id, &"New desc".to_string()).unwrap();
        let shelf_data = get_shelf_data().unwrap();
        assert_eq!(shelf_data.commands[0].description, "New desc");

        edit_command_string(&id, &"echo new".to_string()).unwrap();
        let shelf_data = get_shelf_data().unwrap();
        assert_eq!(shelf_data.commands[0].command, "echo new");
    }

    #[test]
    fn test_delete_command() {
        let _guard = setup_test_env();

        save_command("echo test".to_string(), None, None).unwrap();
        let shelf_data = get_shelf_data().unwrap();
        assert_eq!(shelf_data.commands.len(), 1);
        let id = shelf_data.commands[0].id;

        delete_command(&id).unwrap();
        let shelf_data = get_shelf_data().unwrap();
        assert_eq!(shelf_data.commands.len(), 0);
    }
}
