<p align="center">
  <img src="https://raw.githubusercontent.com/mlavrinenko/mindtape/main/www/assets/images/logo.svg" alt="MindTape" width="120" />
</p>

<h1 align="center">MindTape</h1>

<p align="center">
  A <a href="LICENSE">MIT</a>-licensed, daemon-free file-based task tracker powered by <a href="https://typst.app/">Typst</a>.
</p>

<p align="center">
  <a href="https://mlavrinenko.github.io/mindtape/">Documentation</a> · <a href="https://mlavrinenko.github.io/mindtape/quickstart/">Quickstart</a> · <a href="https://mlavrinenko.github.io/mindtape/comparison/">Comparison</a>
</p>

---

MindTape tracks tasks as one-task-per-file Typst artifacts. Each file declares a
task with `#show: task.with(...)`; MindTape live-evaluates the configured roots on
every command — no daemon, no database, no index you maintain. A task's `id`
identifies it in listings, filters and links-by-hand, and defaults to the
filename stem. Links target files, so renaming a task without updating its
dependents fails `mt check` instead of dangling silently.

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

From crates.io:

```bash
cargo install mindtape-cli
```

Or grab a prebuilt binary for Linux, macOS or Windows from the
[Releases page](https://github.com/mlavrinenko/mindtape/releases), or run it
without installing:

```bash
nix run github:mlavrinenko/mindtape -- --help
```

Building from source needs only a Rust toolchain (`cargo build --release`
produces `target/release/mt`); the `nix develop` shell adds the full test and
lint toolchain used by `just check`.

Linux, macOS and Windows each get a prebuilt binary per release;
[Internals](https://mlavrinenko.github.io/mindtape/internals/) covers the two
Windows seams and what CI verifies where.

### Set up, once per machine

```bash
mt install
```

Task files begin with `#import "@local/mindtape:0.2.0"`, and Typst resolves
`@local` packages from a directory on your machine, not from the repository. So
until `mt install` puts the prelude there, every task file fails to evaluate —
including in a repository someone else set up. `mt install` also seeds the
global defaults and the JSON Schema that gives editors config completion. Any
command that hits an uninstalled prelude says so and names this command.

### Set up, once per project

```bash
mt init
```

Writes `.mindtape/config.toml`. With a terminal and no arguments it asks a short
questionnaire; `--preset <name>` skips it and writes a named starter shape
(`mt init --list-presets`). See
[Quickstart](https://mlavrinenko.github.io/mindtape/quickstart/) for what each
preset is for and
[Configuration](https://mlavrinenko.github.io/mindtape/config/) for the
four-layer merge both commands write into.

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
  `--color`, `--hyperlinks`, `-v`), grouped. Includes `mt find TERM…` (sugar
  over `contains()`) and `mt ls --format path`/`path0` (a bare path list for
  piping into another program, no `jq` needed).
- [Filter language](https://mlavrinenko.github.io/mindtape/filters/) — the
  fields, operators, and functions `mt ls` / `mt rm` accept.
- [Templates & slots](https://mlavrinenko.github.io/mindtape/templates/) —
  every way `mt add` can fill a task from a Typst template.
- [Configuration](https://mlavrinenko.github.io/mindtape/config/) — the
  four-layer config merge, every `.mindtape/config.toml` section, and stem
  naming for minted filenames.
- [Import & export](https://mlavrinenko.github.io/mindtape/import-export/) —
  `mt import` plus `jq` adapters for Taskwarrior and GitHub Issues. There is no
  `mt export` command; the tasks are already plain files, and the page shows
  how to reshape them into another tool's format.
- [For AI agents](https://mlavrinenko.github.io/mindtape/agents/) —
  structured JSON, slot schemas, and driving `mt` from outside the project
  tree.
- [Internals](https://mlavrinenko.github.io/mindtape/internals/) — the crate
  layout, the content-hash cache that keeps live evaluation cheap (a warm
  `mt ls` over 2000 tasks lands at 0.20 s), and what differs per platform.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## Acknowledgements

The logo uses a brain icon from [Lucide](https://lucide.dev) (ISC license).
