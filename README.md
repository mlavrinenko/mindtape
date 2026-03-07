# MindTape

A file-based task tracker powered by [Typst](https://typst.app/).

Write `.typ` files to manage tasks across your projects. MindTape watches your folders,
evaluates the Typst files, and indexes everything into a searchable SQLite database.

## Why Typst?

Unlike Markdown, Typst files can import each other, define typed variables, and be
statically checked. Your task data is structured, validated, and composable.

```typ
#import "@local/mindtape:0.1.0": *

= Sprint 12

- [ ] implement auth #due(2026, 3, 1) #high #tag("backend") #id("auth-123")
- [ ] research options #start(2026, 3, 15) #medium #tag("backend") #id("research-789")
- [x] design mockups #tag("design") #id("design-456")
- [ ] write tests #low
```

## Installation

### Nix flake (NixOS / Home Manager)

```nix
# flake inputs:
mindtape.url = "github:mlavrinenko/mindtape";
mindtape.inputs.nixpkgs.follows = "nixpkgs";

# NixOS module:
imports = [ mindtape.nixosModules.default ];

services.mindtape = {
  enable = true;
  user = "tank";
  group = "users";
  watchPaths = [
    { path = "/home/tank/notes"; }
  ];
};
```

### From source

```bash
git clone https://github.com/mlavrinenko/mindtape.git
cd mindtape
nix develop   # or install Rust toolchain manually
cargo build --release
```

### Typst library

Install the MindTape prelude (`due()`, `start()`, `id()`, `tag()`, `rank()`, `high`, `medium`, `low`) for your Typst files:

```bash
mindtape init          # writes lib to ~/.local/share/typst/packages/local/mindtape/0.1.0/
mindtape init --force  # overwrite existing files
```

## Usage

```bash
# Evaluate a single file
mindtape tasks.typ
mindtape tasks.typ --due -n 5      # tasks with due dates, limit to 5

# Watch folders and build the index
mindtape watch                     # uses ~/.config/mindtape/config.toml
mindtape watch --config my.toml    # custom config

# Query the index
mindtape list                              # pending tasks
mindtape list --status all --tag backend   # filter by status and tag
mindtape list --due-before 2026-04-01      # due soon
mindtape list --start-after 2026-03-01     # started after date
mindtape list --rank-min 50                # medium priority and above
mindtape list --sort rank:desc -10         # by priority, limit 10
mindtape list --json                       # JSON output

# Modify tasks (writes back to .typ files)
mindtape check <task-id>                   # toggle checkbox
mindtape set <task-id> --due 2026-04-01    # set due date
mindtape set <task-id> --start 2026-03-01  # set start date
mindtape set <task-id> --rank 75           # set rank
mindtape set <task-id> --add-tag urgent    # add tag

# Inspect
mindtape status                    # index statistics
mindtape files                     # indexed files
mindtape deps                      # file dependency graph
mindtape id                        # generate a new UUIDv7 task ID
```

## Configuration

```toml
# ~/.config/mindtape/config.toml

[database]
path = "~/.local/share/mindtape/index.db"

[[watch]]
path = "~/projects/myproject"
recursive = true
```

## Docs

- [Contributing](CONTRIBUTING.md) — development workflow, testing, commit style
- [Design](docs/design/) — architecture deep-dives (eval, store, watcher, write-back)

## License

[MIT](LICENSE)
