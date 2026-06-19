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
  estimate: (days: 1,),
  tags: ("backend",),
  relates-to: (link("write-docs.typ"),),
)

== Summary
Wire up the login flow.
```

Everything scalar or enum — `status`, `priority`, `grade`, `kind`, `estimate`
(an opaque `unit -> quantity` dict with a per-project unit allow-list), dates,
tags, and task links — is a generic core field constrained by
`.mindtape/config.toml`. Tools that build on MindTape add their own fields
through the `x-<ns>` extension seam, which MindTape carries opaquely.

## Installation

```bash
git clone https://github.com/mlavrinenko/mindtape.git
cd mindtape
nix develop                 # dev shell with the full toolchain
cargo build --release       # produces target/release/mindtape
```

Install the Typst prelude so `#import "@local/mindtape:0.2.0"` resolves (once
per machine), then scaffold a project (once per project):

```bash
mindtape install-lib        # installs @local/mindtape into Typst packages dir
mindtape init               # writes .mindtape/config.toml in the current dir
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
mindtape list 'stale(30)'                     # unreviewed for over 30 days
mindtape list --sort priority:asc -n 10      # sort + limit

# Re-affirm a task as still relevant (stamps reviewed: with today's date)
mindtape review implement-auth               # by slug, relative, or absolute path

# Validate every task against the project config
mindtape check                               # enum / required / bounds / links

# Evaluate a single task file and print it
mindtape eval tasks/implement-auth.typ
mindtape eval tasks/implement-auth.typ --json
```

### Global flags

```
--color <auto|always|never>   Colour output (default: auto).
                              auto — colour only when stdout is a TTY and
                                     NO_COLOR is not set.
                              always — force ANSI colour sequences.
                              never  — suppress all colour.
                              See https://no-color.org/ for the NO_COLOR spec.
-v / --verbose                Increase verbosity (-v info, -vv debug, -vvv trace).
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

# Allowed estimate units (free labels; never converted or compared). An estimate
# is a unit dict, e.g. `estimate: (days: 5, hours: 5)` or `estimate: (points: 8)`.
# Omit this section to accept any unit.
[tasks.estimate]
units = ["minutes", "hours", "days", "weeks"]

[query]
default_filter = "status != \"done\""
```

### Filter language

Comparable fields: `status`, `priority`, `grade`, `kind`, `title`, `id`, `due`,
`start`, `reviewed`, `file`. Operators: `==`, `!=`, `<`, `>`, `<=`, `>=`, `&&`,
`||`, `!`, parentheses. Functions: `has(field)`, `miss(field)`,
`has_tag("name")`, `contains(field, "substr")`, `search("query")`, `stale(N)`
(re-affirmed — or, lacking `reviewed`, last status-changed — more than N days
ago). The opaque `estimate` dict supports presence only (`has(estimate)`), not
comparison.

## Architecture

A three-crate workspace:

- `typst-harvest` — a package-agnostic, eval-only Typst World plus
  labelled-metadata-marker harvesting. A reusable substrate for any file-based
  tool that uses Typst as a data medium.
- `mindtape` — the domain library: config, the task item model, the filter
  language, and config-driven validation.
- `mindtape-cli` — the thin `mindtape` command-line binary.
- `typst-cst` — Parse-plane structure extraction for Typst — heading tree with source spans, no eval, no World, no IO.
- `typst-edit` — In-place editing for Typst source — parse-plane span queries and an atomic, non-overlapping changeset over typst-syntax; no eval, no World, no IO.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## Acknowledgements

The logo uses a brain icon from [Lucide](https://lucide.dev) (ISC license).
