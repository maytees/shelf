mod cmd;
mod config;
mod fuzzy;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use cmd::{
    add_tag, copy_command, delete_command, edit_command_string, edit_description, fuzzy_search,
    list_commands, remove_tag, run_command, save_command,
};
use config::{get_config_dir, get_config_path, load_config};

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
    /// Display config information/paths. In case you need to find
    /// your config folder.
    Config,
    /// Save a command
    #[command(alias = "save")]
    Stack {
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
        /// In addition to ID, and command, display tags, and description
        #[arg(short, long, required = false)]
        verbose: bool,
        /// Reverse the order of the listed commands
        #[arg(short, long, required = false)]
        reverse: bool,
        /// Limit the order of the listed commands.
        #[arg(short, long)]
        limit: Option<u32>,
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
        /// Copy a selected command rather than run
        #[arg(short, long, required = false)]
        copy: bool,
    },
    /// Delete a saved command by ID
    #[command(name = "delete", alias = "del")]
    Delete { id: u32 },
    /// Remove a tag from a saved command
    Rmtag { id: u32, tag: String },
    /// Add a tag to a saved command
    Addtag { id: u32, tag: String },
    /// Edit the description of a saved command
    #[command(name = "editdesc", alias = "edesc")]
    EditDesc { id: u32, description: String },
    /// Edit the command string of a saved command
    #[command(name = "editcommand", alias = "ecmd")]
    EditCommand {
        id: u32,
        #[arg(required = true, allow_hyphen_values = true, trailing_var_arg = true)]
        command: Vec<String>,
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
        Some(Commands::Stack {
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
        Some(Commands::List {
            verbose,
            reverse,
            limit,
        }) => {
            list_commands(
                &(config.auto_verbose.unwrap_or(false) || *verbose),
                reverse,
                limit,
            )?;
        }
        Some(Commands::Run { id, copy }) => {
            if *copy {
                return copy_command(id);
            }

            // Run command
            return run_command(id);
        }
        Some(Commands::Fuzz { copy }) => return fuzzy_search(copy),
        Some(Commands::Delete { id }) => {
            delete_command(id)?;
        }
        Some(Commands::Rmtag { id, tag }) => {
            remove_tag(id, tag)?;
        }
        Some(Commands::Addtag { id, tag }) => {
            add_tag(id, tag)?;
        }
        Some(Commands::EditDesc { id, description }) => {
            edit_description(id, description)?;
        }
        Some(Commands::EditCommand { id, command }) => {
            edit_command_string(id, &command.join(" "))?;
        }
        None => {}
    }

    Ok(())
}
