<p align="center">
  <img src="www/logo.svg" alt="MindTape" width="120" />
</p>

<h1 align="center">MindTape</h1>

<p align="center">
  A <a href="LICENSE">MIT</a>-licensed file-based task tracker powered by <a href="https://typst.app/">Typst</a>.
</p>

---

Write `.typ` files, and MindTape indexes them into a searchable SQLite database.
Unlike Markdown, Typst files can import each other, define typed variables, and be
statically checked — your task data is structured, validated, and composable.

```typ
#import "@local/mindtape:0.1.0": *

= Sprint 12

- [ ] implement auth  #due(2026, 3, 1) #high #tag("backend") #id("01JNW73M")
- [ ] research options #start(2026, 3, 15) #medium #tag("backend")
- [x] design mockups   #tag("design")
- [ ] write tests       #low
```

## Installation

### Nix flake (NixOS)

See [`nix/install-example.nix`](nix/install-example.nix) for a full NixOS module example.

### From source

```bash
git clone https://github.com/mlavrinenko/mindtape.git
cd mindtape
nix develop   # or install Rust toolchain manually
cargo build --release
```

After building, install the Typst prelude so `#import "@local/mindtape:0.1.0"` works:

```bash
mindtape init          # writes lib to ~/.local/share/typst/packages/local/mindtape/0.1.0/
mindtape init --force  # overwrite existing files
```

## Usage

```bash
# Evaluate a single file
mindtape tasks.typ

# Watch folders and build the index
mindtape watch                     # uses ~/.config/mindtape/config.toml
mindtape watch --config my.toml

# Query the index
mindtape list                                         # pending tasks
mindtape list --status all --filter 'has_tag("backend")'
mindtape list --filter 'has(due) && due < "2026-04-01"'
mindtape list --filter 'search("auth")'               # full-text search
mindtape list --sort rank:desc -n 10                   # top priority
mindtape list --json                                   # JSON output

# Modify tasks (writes back to .typ files)
mindtape check <id>                    # toggle checkbox
mindtape set <id> --due 2026-04-01     # set due date
mindtape set <id> --add-tag urgent     # add tag

# Inspect
mindtape status                        # index statistics
mindtape files                         # indexed files
mindtape deps                          # file dependency graph
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## Acknowledgements

The logo uses a brain icon from [Lucide](https://lucide.dev) (ISC license).
