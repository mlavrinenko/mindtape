<p align="center">
  <img src="https://raw.githubusercontent.com/mlavrinenko/mindtape/main/www/logo.svg" alt="MindTape" width="120" />
</p>

<h1 align="center">MindTape</h1>

<p align="center">
  A <a href="LICENSE">MIT</a>-licensed, daemon-free file-based task tracker powered by <a href="https://typst.app/">Typst</a>.
</p>

---

MindTape tracks tasks as one-task-per-file Typst artifacts. Each file declares a
task with `#show: task.with(...)`; MindTape live-evaluates the configured roots on
every command — no daemon, no database, no global index. Identity is the
filename stem, so links between tasks break loudly when a file is renamed.

```typ
#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Implement authentication",
  status: wip(2026, 6, 14),
  priority: "high",
  estimate-hours: 8,
  tags: ("backend",),
  relates-to: (link("write-docs.typ"),),
)

== Summary
Wire up the login flow.
```

Everything scalar or enum — `status`, `priority`, `grade`, `kind`,
`estimate-hours`, dates, tags, and task links — is a generic core field
constrained by `.mindtape/config.toml`. Tools that build on MindTape add their
own fields through the `x-<ns>` extension seam, which MindTape carries opaquely.

## Installation

```bash
git clone https://github.com/mlavrinenko/mindtape.git
cd mindtape
nix develop                 # dev shell with the full toolchain
cargo build --release       # produces target/release/mindtape
```

Install the Typst prelude so `#import "@local/mindtape:0.2.0"` resolves, and
scaffold a project:

```bash
mindtape init               # installs the lib + writes .mindtape/config.toml
mindtape init --no-lib      # only scaffold config (lib already installed)
```

## Usage

```bash
# List tasks under the configured roots (applies query.default_filter)
mindtape list
mindtape list --all                          # ignore the default filter
mindtape list 'has_tag("backend")'           # filter expression
mindtape list 'status != "done" && priority == "high"'
mindtape list 'has(due) && due < "2026-08-01"'
mindtape list 'search("auth")' --json        # JSON output
mindtape list --sort priority:asc -n 10      # sort + limit

# Validate every task against the project config
mindtape check                               # enum / required / bounds / links

# Evaluate a single task file and print it
mindtape eval tasks/implement-auth.typ
mindtape eval tasks/implement-auth.typ --json
```

### Configuration

`.mindtape/config.toml` white-lists what MindTape constrains; omit a section to
leave it unconstrained.

```toml
[tasks]
roots = ["tasks"]

[tasks.status]
values   = ["proposed", "accepted", "wip", "done", "cancelled", "failed"]
terminal = ["done", "cancelled", "failed"]

[tasks.fields.priority]
values   = ["low", "normal", "high"]
required = "non-terminal"          # true | false | "non-terminal"

[tasks.fields.estimate-hours]
type = "number"
min  = 0

[query]
default_filter = "status != \"done\""
```

### Filter language

Fields: `status`, `priority`, `grade`, `kind`, `title`, `id`, `due`, `start`,
`estimate_hours`, `file`. Operators: `==`, `!=`, `<`, `>`, `<=`, `>=`, `&&`,
`||`, `!`, parentheses. Functions: `has(field)`, `miss(field)`,
`has_tag("name")`, `contains(field, "substr")`, `search("query")`.

## Architecture

A three-crate workspace:

- `typst-harvest` — a package-agnostic, eval-only Typst World plus
  labelled-metadata-marker harvesting. A reusable substrate for any file-based
  tool that uses Typst as a data medium.
- `mindtape` — the domain library: config, the task item model, the filter
  language, and config-driven validation.
- `mindtape-cli` — the thin `mindtape` command-line binary.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## Acknowledgements

The logo uses a brain icon from [Lucide](https://lucide.dev) (ISC license).
