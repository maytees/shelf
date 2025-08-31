# Shelf

Your personal CLI bookshelf for storing and recalling useful commands.
Shelf allows you to store commands in their respective collections, created by you.

**No more _"I know I used this command last month, but what was it again?"_ moments**

## Installing with Cargo

View the crates.io page [here](https://crates.io/crates/shelf-cli)

Requirements:

- Cargo/Rust

1. Cargo install

Install the `shelf` binary:

```bash
cargo install shelf-cli
```

**The install might give you a warning:**

Be sure to add `/path/to/.cargo/bin` _(it will tell you)_ to your PATH to be able to run the installed binaries.

## Building from source

Requirements:

- Rust
- Git (ofc)

1. Clone the repo

```bash
git clone https://github.com/maytees/shelf
```

2. Build it

```bash
cargo build
```

## How to use

### Saving a command

This will store a command to the global collection.

```bash
shelf stack -d "Prints out HOME env var" echo $HOME
```

To stack a command with tags, specifiy with the -t flag.

```bash
shelf stack -d "Builds a NextJS Project" -t nextjs,npx,npm npx next build
```

### Command Templates

Shelf supports command templates using `{{parameter}}` syntax. When you save a command with double curly brace parameters, it becomes a template that will prompt for values when run.

#### Saving templates

```bash
# SSH template
shelf stack -d "SSH to any server" ssh {{user}}@{{host}}

# Docker template
shelf stack -d "Run container interactively" docker run -it {{image}} {{command}}

# Git clone with branch
shelf stack -d "Clone specific branch" git clone -b {{branch}} {{repo}}
```

#### Running templates

When you run a template command, Shelf will prompt for each parameter:

```bash
shelf run 5
# This is a template command. Please provide values:
# Enter user: admin
# Enter host: myserver.com
# Executes: ssh admin@myserver.com
```

#### Template features

- **Automatic detection**: Any command with `{{param}}` becomes a template
- **Unique parameters**: Same parameter name used multiple times gets same value
- **Works everywhere**: Templates work with `run`, `copy`, and fuzzy search
- **No conflicts**: Regular `{braces}`, `$variables`, and `<redirections>` work normally
- **Literal braces**: Use `\{{text}}` to save literal `{{text}}` without templating

### Listing commands

By default, this will act similar to shell history, and print out saved commands in order.
Use the `--verbose` flag to display the _command_ description, and _tags_
in addition to just the id and command

```bash
shelf list
```

#### Options

```
-v, --verbose        In addition to ID, and command, display tags, and description

-r, --reverse        Reverse the order of the listed commands

-l, --limit <LIMIT>  Limit the order of the listed commands
```

### Running a command

> [!TIP]
> If a command is saved with an environment variable, the
> variable will be evaluated when you run the command. If you wish to
> evaluate the variable when you stack the command, use your shell's
> method of entering variables as plain strings. For example in zsh
> you add a `\` before the variable: `\$HOME`.

Currently, there are two ways to _fetch_ commands in shelf:

1. Running
2. Copying to clipboard

#### Running

To run a command, first find the `id` of the command via `shelf list`

```bash
shelf run <ID>
```

To run a command AND copy it to clipboard:

```bash
shelf run -c <ID>
```

#### Copying to clipboard

Copy a command to clipboard without running it:

```bash
shelf copy <ID>
# or use the short alias
shelf c <ID>
```

### Deleting commands

Remove a saved command permanently:

```bash
shelf delete <ID>
```

### Editing commands

#### Managing tags

Add a tag to an existing command:

```bash
shelf addtag <ID> <TAG>
```

Remove a tag from a command:

```bash
shelf rmtag <ID> <TAG>
```

#### Editing descriptions and commands

Update the description of a saved command:

```bash
shelf editdesc <ID> "New description here"
```

Edit the command string itself:

```bash
shelf editcommand <ID> new command here
```

### Fuzzy searching

You are able to fuzzy search commands to either run them or copy them.

#### Run via fuzzy search

```bash
shelf fuzz
```

#### Copying to clipbaord via fuzzy search

```bash
shelf fuzz -c
```

![image](https://github.com/user-attachments/assets/84e0ccb0-e6cf-455f-ad16-967d5607e7c6)

## Config

The configuration for shelf is currently very limited. Here is what is configurable at the moment:

### Auto Verbose

Automatically outputs verbose list of commands **(default: false)**

```toml
auto_verbose = false
```

## Environment Variables

Shelf supports the following environment variables to customize file locations:

### SHELF_DATA_DIR

Override the default data directory where commands are stored:

```bash
export SHELF_DATA_DIR="/custom/path"
# Commands will be stored in /custom/path/cmds.toml
```

### SHELF_CONFIG_DIR

Override the default config directory:

```bash
export SHELF_CONFIG_DIR="/custom/config/path"
# Config will be stored in /custom/config/path/config.toml
```

These are particularly useful for testing or when you want to use different shelf instances.

## Shell Completion

Shelf supports shell completion for bash, zsh, fish, and PowerShell.

### Generate completion scripts

```bash
# For bash
shelf completion bash > ~/.local/share/bash-completion/completions/shelf

# For zsh  
shelf completion zsh > ~/.zsh/completions/_shelf

# For fish
shelf completion fish > ~/.config/fish/completions/shelf.fish

# For PowerShell
shelf completion powershell > shelf.ps1
```

### Setup instructions

**Bash:**
```bash
# Add to ~/.bashrc
source ~/.local/share/bash-completion/completions/shelf
```

**Zsh:**
```bash
# Add to ~/.zshrc
fpath=(~/.zsh/completions $fpath)
autoload -U compinit && compinit
```

**Fish:**
Fish will automatically load completions from `~/.config/fish/completions/`

**PowerShell:**
```powershell
# Add to your PowerShell profile
. ./shelf.ps1
```

## Testing

Run the test suite with:

```bash
cargo test
```

The tests use temporary directories and environment variable isolation to avoid interfering with your personal shelf data.

## Todo

- [x] Save
  - [x] Stack globally
  - [ ] Stack to a collection
  - [x] Stack with tags
- [x] List
  - [ ] List a collection
  - [x] Reverse flag
  - [x] Limit flag
  - [x] Verbose flag
    - [x] Description
    - [x] Tags
    - [ ] Collection
- [x] Run commands
  - [x] Via search
  - [x] Via id (similar to shell history)
  - [x] Copy command via flag
- [x] Edit commands
  - [x] Delete commands
  - [x] Add tags to existing commands
  - [x] Remove tags from commands
  - [x] Edit command descriptions
  - [x] Edit command strings
- [ ] Search
  - [x] Fuzzy search
  - [ ] Search by tag
  - [ ] Search a collection by tag
- [ ] Shell history integration
  <!--- (note: in the short term, this can be achieved with something akin to `shelf stack -d "command" $(history $NUMBER_TO_STORE_IF_APPLICABLE | tail -n 1 | awk '{for (i=2; i<NF; i++) printf $i " "; print $NF}')`)-->
- [x] Colored output (for readability)
- [ ] Run Command on store
  - [ ] Save command output
    - [ ] Store x lines of output
- [ ] Much more...
