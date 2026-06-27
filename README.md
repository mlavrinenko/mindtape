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
  priority: ice(impact: 8, confidence: 0.9, ease: 7),
  estimate: (days: 1,),
  tags: ("backend",),
  relates-to: (link("write-docs.typ"),),
)

== Summary
Wire up the login flow.
```

Everything scalar or enum — the `status` legend, the numeric `priority` and
`difficulty` scales, `kind`, `estimate` (an opaque `unit -> quantity` dict with
a per-project unit allow-list), dates, tags, and task links — is a generic core
field constrained by `.mindtape/config.toml`. Tools that build on MindTape add
their own fields through the `x-<ns>` extension seam, which MindTape carries
opaquely.

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
# File a new task from a Typst slot template (fills slots, injects status + id)
mindtape add --describe                       # slot schema (--describe=json for agents)
mindtape add --set title="Wire OAuth" --set priority=7      # raw --set values
echo '{"title":"Wire OAuth","priority":7}' | mindtape add -f -   # JSON values
mindtape add cleanup kitchen                  # positional title → cleanup-kitchen.typ (or pipe stdin / MINDTAPE_ADD_<SLOT> env)
mindtape add --edit                           # fill the slots in your editor (YAML buffer)
mindtape add -t bug --name oauth-bug --to auth # named template, explicit stem, nested under <root>/auth/

# List tasks under the configured roots (applies ls.default-filter)
mindtape ls
mindtape ls --all                          # ignore the default filter
mindtape ls 'has_tag("backend")'           # filter expression
mindtape ls 'status != "done" && priority >= 7' --json   # filter + JSON output
mindtape ls 'stale(30)'                     # unreviewed for over 30 days
mindtape ls ready                           # deps all done — startable now
mindtape ls --sort priority:asc -n 10      # sort + limit

# Re-affirm a task as still relevant (stamps reviewed: with today's date)
mindtape review implement-auth               # by slug, relative, or absolute path

# Open a task in your editor ([go] editor → $VISUAL → $EDITOR; {task} → path)
mindtape go 1b                               # by handle, slug, or path

# Set a task's status dated today; each legend value is also a shortcut verb
mindtape flip done implement-auth            # canonical; or `mindtape done 1b 2c` (== flip done)
mindtape flip blocked 1b 2c -m "note"        # batch + note; `mindtape flip --list` shows the verbs

# Validate every task against the project config
mindtape check                               # enum / required / bounds / links / cycles

# Evaluate a single task file and print it (--json for machine output)
mindtape eval tasks/implement-auth.typ

# Retire tasks: mint a swh:1:rev tombstone, rewrite inbound link()s, do not commit
mindtape rm                                  # [rm] default-filter (built-in: terminal)
mindtape rm implement-auth                   # by ref: slug, path, or list handle (1v)
mindtape rm 'status == "cancelled"'          # by filter — every match
mindtape rm terminal --dry-run               # preview without writing
```

### Global flags

```
--color <auto|always|never>   Colour output (default auto: colour when stdout is
                              a TTY and NO_COLOR unset). always forces; never
                              suppresses. NO_COLOR spec: https://no-color.org/
-v / --verbose                Increase verbosity (-v info, -vv debug, -vvv trace).
```

### Configuration

`.mindtape/config.toml` white-lists what MindTape constrains; omit a section to
leave it unconstrained.

```toml
[tasks]
roots = ["tasks"]
# Table-form roots add a display name and folder-derived tags (`[[tasks.roots]]`
# with `name` + `derive-tags`, plus a project-wide `[tasks.derive-tags]` policy):
# nested tasks inherit tags from their folders. See default-config.toml.

# Status legend: legal values in lifecycle order, each with an optional glyph
# and colour. `terminal = true` ends a task (required fields relax once terminal).
[[tasks.status]]
value = "wip"
glyph = "🔨"
color = "yellow"
[[tasks.status]]
value = "done"
glyph = "✅"
color = "green"
terminal = true

# Priority: a number on a scale, classified into labelled bands. `required` is
# true | false | "non-terminal". Difficulty is the sibling, classified by anchors.
# `framework` (optional) computes the score from typed inputs instead of a hand-
# set int: built-ins ice/rice/wsjf/eisenhower/moscow ship baked-in; a task then
# writes `priority: ice(impact: 8, confidence: 0.9, ease: 7)`. `allow-manual =
# false` is strict (no raw ints). A framework's range is checked to fit the scale
# at config load. Define or override one under `[tasks.priority.frameworks.<name>]`.
[tasks.priority]
kind = "int"                       # int | real
min  = 0
max  = 9
required = "non-terminal"
framework = "ice"                  # optional; omit for hand-set integers
allow-manual = false               # strict: every task scores through the framework
[[tasks.priority.bands]]
label = "high"
min   = 7
max   = 9
glyph = "🔴"
color = "red"

# Allowed estimate units (free labels; never converted or compared). An estimate
# is a unit dict, e.g. `estimate: (days: 5, hours: 5)` or `estimate: (points: 8)`.
# Omit this section to accept any unit.
[tasks.estimate]
units = ["minutes", "hours", "days", "weeks"]

[ls]
default-filter = "status != \"done\""
```

### Filter language

Comparable fields: `status`, `priority`, `difficulty`, `kind`, `title`, `id`,
`due`, `start`, `reviewed`, `file`. `priority` and `difficulty` compare
numerically against a number (e.g. `priority >= 7`). Operators: `==`, `!=`, `<`,
`>`, `<=`, `>=`, `&&`, `||`, `!`, parentheses. Functions: `has(field)`, `miss(field)`,
`has_tag("name")`, `contains(field, "substr")`, `search("query")`, `stale(N)`
(re-affirmed — or, lacking `reviewed`, last status-changed — more than N days
ago). The opaque `estimate` dict supports presence only (`has(estimate)`), not
comparison. Dependency predicates `ready` and `blocked` (bare or as `ready()` /
`blocked()`) classify a task by its `depends-on` targets: ready when every direct
dependency is terminal, blocked otherwise. `list` floats ready work to the top by
default. Empty columns auto-hide; `[ls.columns]` / `columns-order` tune it. The `terminal`
predicate (bare or `terminal()`) matches a task in a terminal status; it is the
built-in `[rm] default-filter`.

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
