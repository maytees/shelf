# Shelf

Your personal CLI bookshelf for storing and recalling useful commands.
Shelf allows you to store commands in their respective collections, created by you.

**No more _"I know I used this command last month, but what was it again?"_ moments**

## Building from source

No package manager **_yet_**...

Requirements:

- Rust

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
shelf save -d "Prints out HOME env var" echo $HOME
```

### Listing commands

By default, this will act similar to shell history, and print out saved commands in order.
Use the `--verbose` flag to display the command description _(there will be more info in the future)_

```bash
shelf list
```

![image](https://github.com/user-attachments/assets/84e0ccb0-e6cf-455f-ad16-967d5607e7c6)

## Todo

- [ ] Save
  - [x] Save globally
  - [ ] Save to a collection
  - [ ] Save with tags
- [x] List
  - [ ] List a collection
  - [x] Reverse flag
  - [x] Verbose flag
- [ ] Run commands
  - [ ] Via search
  - [ ] Via id (similar to shell history)
- [ ] Search
  - [ ] Fuzzy search
  - [ ] Search by tag
  - [ ] Search a collection by tag
- [ ] Shell history integration
- [ ] Colored output (for readability)
- [ ] Much more...
