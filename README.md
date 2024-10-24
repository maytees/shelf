# Shelf

Your personal CLI bookshelf for storing and recalling useful commands.
Shelf allows you to store commands in their respective collections, created by you.
**No more "I know I used this command last month, but what was it again?" moments**

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

<!-- TODO: List commands -->

## Todo

- [ ] Save
  - [x] Save globally
  - [ ] Save to a collection
  - [ ] Save with tags
- [ ] List
  - [ ] List a collection
- [ ] Search
  - [ ] Fuzzy search
  - [ ] Search by tag
  - [ ] Search a collection by tag
- [ ] Shell history integration
- [ ] Colored output (for readability)
- [ ] Much more...
