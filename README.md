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
Use the `--verbose` flag to display command descriptions *(there will be more info in the future)*

```bash
shelf list
```

<img width="761" alt="image" src="https://github.com/user-attachments/assets/13ab426c-4541-4a10-a7eb-e6fd7eeeaf2b">


## Todo

- [ ] Save
  - [x] Save globally
  - [ ] Save to a collection
  - [ ] Save with tags
- [x] List
  - [ ] List a collection
  - [x] Reverse flag
  - [x] Verbose flag
- [ ] Search
  - [ ] Fuzzy search
  - [ ] Search by tag
  - [ ] Search a collection by tag
- [ ] Shell history integration
- [ ] Colored output (for readability)
- [ ] Much more...
