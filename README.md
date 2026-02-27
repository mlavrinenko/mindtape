# MindTape

A file-based task tracker powered by [Typst](https://typst.app/).

Write `.typ` files to manage tasks across your projects. MindTape watches your folders,
evaluates the Typst files, and indexes everything into a searchable database.

## Why Typst?

Unlike Markdown, Typst files can import each other, define typed variables, and be
statically checked. This means your task data is structured, validated, and composable.

```typ
#import "@local/mindtape:0.1.0": due, tag, id

= Sprint 12

- [ ] implement auth #due(datetime(year: 2026, month: 3, day: 1)) #tag("backend") #id("auth-123")
- [x] design mockups #tag("design") #id("design-456")
- [ ] write tests

#let note = "blocked on API spec from team B"
```

### Installing the MindTape Library

To use `#import "@local/mindtape:0.1.0"` in your Typst files globally:

```bash
just install-lib
```

This creates a symlink at `~/.local/share/typst/packages/local/mindtape/0.1.0/` pointing
to the `lib/` directory. The library works with both the Typst CLI (`typst compile`) and
MindTape evaluation.

To uninstall:

```bash
just uninstall-lib
```

## How It Works

1. Configure folders to watch
2. Run `mindtape watch`
3. MindTape evaluates your `.typ` files using the Typst compiler
4. Tasks, properties, and bindings are indexed into SQLite
5. Query with `mindtape list`, filter by status/tag/due date

## NixOS installation

```nix
# inputs:
mindtape.url = "path:/home/tank/projects/home/mindtape";
mindtape.inputs.nixpkgs.follows = "nixpkgs";

# modules list:
mindtape.nixosModules.default

# configuration:
services.mindtape = {
  enable = true;
  user = "tank";    # run as your user instead of a system user
  group = "users";
  watchPaths = [
    { path = "/home/tank/notes"; }
  ];
};
```

## Docs

- [Design](docs/DESIGN.md) — architecture, data model, technical decisions
- [Roadmap](docs/ROADMAP.md) — MVP scope, milestones, non-goals

## TODO

- `mindtape project` - create project by template?

## Future / Ideas

- TUI (ratatui)
- DuckDB as alternative store backend
- Typst package published to `@preview` for `due`, `tag`, etc.
- Custom user-defined task properties
- Recurring tasks
- Task dependencies / blocking relationships
- Notifications (desktop, email)
- Sync across machines (CRDTs, git-based)
- `mindtape init` scaffolding for new projects
- REST API
- Web UI
- Editor integrations (VS Code, Neovim)

## Non-Goals

- MindTape is NOT a Typst renderer — we never produce PDFs or visual output
- MindTape is NOT a general Typst IDE — use tinymist for that
- MindTape does NOT replace Typst files — they are always the source of truth
- MindTape does NOT require internet access for core functionality
