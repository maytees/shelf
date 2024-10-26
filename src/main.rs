mod cmd;
mod config;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use cmd::{copy_command, fuzzy_search, list_commands, run_command, save_command};
use config::{get_config_dir, get_config_path, load_config};
mod fuzzy;

#[derive(Parser)]
#[command(
    version,
    about = "Shelf - Your personal command-line bookshelf for storing and recalling useful commands",
    long_about = "
A lightweight CLI bookshelf for storing and recalling useful commands. No need to dig 
through shell history for that complex Docker command or git operation - just shelf it and find 
it when you need it.

No more \"I know I used this command last month, but what was it again?\" moments.",
    arg_required_else_help(true)
)]
struct ShelfCli {
    /// Subcommand to run e.g save
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Display config information/paths
    Config,
    // TODO: Change save to stack, but keep save as an alias
    /// Save a command
    Save {
        /// Description of the command (optional)
        #[arg(short, long, required = false)]
        description: Option<String>,

        /// Comma seperated tags, no spaces in between
        #[arg(short, long, allow_hyphen_values = true)]
        tags: Option<String>,

        /// The command to save
        #[arg(required = true, allow_hyphen_values = true, trailing_var_arg = true)]
        command: Vec<String>,
    },
    /// List saved commands
    List {
        #[arg(short, long, required = false)]
        verbose: bool,
        #[arg(short, long, required = false)]
        reverse: bool,
    },
    /// Run a command via an id
    Run {
        #[arg(short, long, required = false)]
        copy: bool,
        id: u32,
    },
    /// Fuzzy search your commands
    #[command(alias = "fuzzy")]
    Fuzz {
        #[arg(short, long, required = false)]
        copy: bool,
    },
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
            tags,
        }) => save_command(
            command.join(" "),
            description.clone(),
            if let Some(tags) = tags {
                Some(tags.split(",").map(|s| s.to_string()).collect())
            } else {
                None
            },
        )?,
        Some(Commands::List { verbose, reverse }) => {
            list_commands(verbose, reverse)?;
        }
        Some(Commands::Run { id, copy }) => {
            if *copy {
                return copy_command(id);
            }

            // Run command
            return run_command(id);
        }
        Some(Commands::Fuzz { copy }) => return fuzzy_search(copy),
        None => {}
    }

    Ok(())
}
