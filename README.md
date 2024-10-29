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

#### Copying to clipboard

Copying to clipboard may not seem all that useful right now, but it may come in handy someday,
so it's there for you to use.. :)

```bash
shelf run -c <ID>
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
- [ ] Search
  - [x] Fuzzy search
  - [ ] Search by tag
  - [ ] Search a collection by tag
- [ ] Shell history integration
- [ ] Colored output (for readability)
- [ ] Much more...
