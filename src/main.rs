mod cmd;
mod config;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use cmd::save_command;
use config::{get_config_dir, get_config_path, load_config};

#[derive(Parser)]
#[command(
    version,
    about = "Shelf - Your personal command-line bookshelf for storing and recalling useful commands",
    long_about = "
A lightweight CLI bookshelf for storing and recalling useful commands. No need to dig 
through shell history for that complex Docker command or git operation - just shelf it and find 
it when you need it.

No more \"I know I used this command last month, but what was it again?\" moments."
)]
struct ShelfCli {
    /// Subcommand to run e.g save
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Display config information
    Config,
    /// Save a command
    Save {
        /// Description of the command (optional)
        #[arg(short, long, required = false)]
        description: Option<String>,
        /// The command to save
        #[arg(required = true, allow_hyphen_values = true, trailing_var_arg = true)]
        command: Vec<String>,
    },
    // TODO: List command
}

fn main() -> Result<()> {
    let config_dir = get_config_dir();
    let config_path = get_config_path(&config_dir);

    let config = load_config(&config_dir, &config_path).context("Could not load config!")?;
    let cli = ShelfCli::parse();

    match &cli.command {
        Some(Commands::Config) => {
            println!("{:?} is the config dir", config_dir);
            println!("{:?} is the config path", config_path);
            println!("{:?} is the storage path", config.storage_path);
        }
        Some(Commands::Save {
            description,
            command,
        }) => {
            // Save command
            let _ = save_command(command.join(" "), description.clone());
        }
        None => {}
    }

    Ok(())
}
