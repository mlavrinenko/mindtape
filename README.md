<p align="center">
  <img src="https://raw.githubusercontent.com/mlavrinenko/mindtape/main/www/assets/images/logo.svg" alt="MindTape" width="120" />
</p>

<h1 align="center">MindTape</h1>

<p align="center">
  A <a href="LICENSE">MIT</a>-licensed, daemon-free file-based task tracker powered by <a href="https://typst.app/">Typst</a>.
</p>

<p align="center">
  <a href="https://mlavrinenko.github.io/mindtape/">Documentation</a> · <a href="https://mlavrinenko.github.io/mindtape/quickstart/">Quickstart</a> · <a href="https://mlavrinenko.github.io/mindtape/import-export/">Import &amp; export</a> (there is no <code>mt export</code> — see why)
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
  links: related("write-docs.typ")[same feature],
)

== Summary
Wire up the login flow.
```

Everything scalar or enum is a generic core field constrained by
`.mindtape/config.toml`; tools built on MindTape add their own data through
the `x-<ns>` extension seam, carried through opaquely. See
[Concepts](https://mlavrinenko.github.io/mindtape/concepts/) for the full
model and [Task DSL](https://mlavrinenko.github.io/mindtape/dsl/) for every
field a task can declare.

## Installation

```bash
git clone https://github.com/mlavrinenko/mindtape.git
cd mindtape
nix develop                 # dev shell with the full toolchain
cargo build --release       # produces target/release/mt
```

Set up the user environment once per machine (installs the Typst prelude, seeds the XDG global app config), then scaffold a project (once per project):

```bash
mt install            # @local/mindtape + ~/.config/mindtape/{defaults.toml,config.schema.json}
mt init               # writes .mindtape/config.toml (with a #:schema directive) here
```

`mt init --list-presets` shows the named starter shapes (`flow`, a lifecycle
with enforced transitions, is the default); `--annotate` keeps a preset's
explanatory comments. See
[Quickstart](https://mlavrinenko.github.io/mindtape/quickstart/) for what
each preset is for and
[Configuration](https://mlavrinenko.github.io/mindtape/config/) for the
four-layer merge `mt install`/`mt init` write into, and the JSON Schema
`mt install` generates for editor completion.

## Usage

One task, start to finish — file it, list it, close it, validate the project:

```bash
mt add --set title="Wire OAuth" --set priority=7   # file a task from the default template
mt ls 'status != "done" && priority >= 7'          # query it with a filter expression
mt flip done wire-oauth                            # set its status, dated today
mt check                                           # validate the whole project
```

That is a sliver of the surface. The docs site has the rest:

- [CLI reference](https://mlavrinenko.github.io/mindtape/cli/) — every `mt`
  command and global flag (`-w`/`--worktree`, `@name` worktree aliases,
  `--color`, `-v`), grouped.
- [Filter language](https://mlavrinenko.github.io/mindtape/filters/) — the
  fields, operators, and functions `mt ls` / `mt rm` accept.
- [Templates & slots](https://mlavrinenko.github.io/mindtape/templates/) —
  every way `mt add` can fill a task from a Typst template.
- [Configuration](https://mlavrinenko.github.io/mindtape/config/) — the
  four-layer config merge, every `.mindtape/config.toml` section, and stem
  naming for minted filenames.
- [For AI agents](https://mlavrinenko.github.io/mindtape/agents/) —
  structured JSON, slot schemas, and driving `mt` from outside the project
  tree.

## Architecture

A two-crate `cargo` workspace — the domain library plus its thin CLI — built
on consumer-blind Typst substrates reusable by any file-based tool:

- `mindtape` — the domain library: config, the task model, the filter language,
  and config-driven validation. `mindtape-cli` is the thin `mt` binary.
- `typst-harvest` (crates.io) — eval-only Typst World plus labelled-metadata-marker
  harvesting (Typst as a data medium).
- `typst-cst` / `typst-edit` (crates.io) — parse-plane structure extraction (heading
  tree with spans) and in-place source editing (span queries + atomic changeset); no
  eval, World, or IO.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## Acknowledgements

The logo uses a brain icon from [Lucide](https://lucide.dev) (ISC license).
