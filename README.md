<!-- Generated from docs/src/readme.typ by `just docs-md`. Do not edit; edit the source. -->

<h1 align="center"><img src="https://raw.githubusercontent.com/mlavrinenko/mindtape/main/www/assets/images/logo.svg" alt="MindTape" width="120" /><br />
 MindTape</h1>

<h4 align="center">A <a href="LICENSE">MIT</a>-licensed, daemon-free file-based task tracker powered by <a href="https://typst.app/">Typst</a>.</h4>

<h4 align="center"><a href="https://mlavrinenko.github.io/mindtape/">Documentation</a> · <a href="https://mlavrinenko.github.io/mindtape/quickstart/">Quickstart</a> · <a href="https://mlavrinenko.github.io/mindtape/comparison/">Comparison</a></h4>

---

MindTape tracks tasks as one-task-per-file Typst artifacts. Each file declares a task with `#show: task.with(...)`; MindTape live-evaluates the configured roots on every command — no daemon, no database, no index you maintain. A task’s `id` identifies it in listings, filters and links-by-hand, and defaults to the filename stem. Links target files, so renaming a task without updating its dependents fails `mt check` instead of dangling silently.

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

Everything scalar or enum is a generic core field constrained by `.mindtape/config.toml`; tools built on MindTape add their own data through the `x-<ns>` extension seam, carried through opaquely. See [Concepts](https://mlavrinenko.github.io/mindtape/concepts/) for the full model and [Task DSL](https://mlavrinenko.github.io/mindtape/dsl/) for every field a task can declare.

## Installation

```bash
cargo install mindtape-cli
```

[Quickstart](https://mlavrinenko.github.io/mindtape/quickstart/) covers the prebuilt binaries, `nix run`, and building from source.

## Usage

One task, start to finish:

```bash
mt install                                         # once per machine: put the Typst prelude where Typst looks
mt init                                            # once per project: write .mindtape/config.toml
mt add --set title="Wire OAuth" --set priority=7   # file a task from the default template
mt ls 'status != "done" && priority >= 7'          # query it with a filter expression
mt flip done wire-oauth                            # set its status, dated today
mt check                                           # validate the whole project
```

The docs site has the rest:

- [Quickstart](https://mlavrinenko.github.io/mindtape/quickstart/) — Install MindTape, scaffold a project, and file your first task.
- [Concepts](https://mlavrinenko.github.io/mindtape/concepts/) — The model behind MindTape: files as tasks, live evaluation, identity by filename, and one extension seam.
- [Task DSL](https://mlavrinenko.github.io/mindtape/dsl/) — Every field a task file can declare, and what the prelude versus the Rust layer enforce.
- [Templates & slots](https://mlavrinenko.github.io/mindtape/templates/) — How mt add scaffolds a task from a Typst template whose values are slot() calls.
- [Derived tags](https://mlavrinenko.github.io/mindtape/derived-tags/) — Tasks inherit tags from their location under a root, without those tags ever being written to the file.
- [CLI reference](https://mlavrinenko.github.io/mindtape/cli/) — Every mt command, grouped, with the deep topics linked out to their own pages.
- [Filter language](https://mlavrinenko.github.io/mindtape/filters/) — The expression language mt ls and mt rm use to select tasks: fields, operators, functions, and dependency predicates.
- [Priority](https://mlavrinenko.github.io/mindtape/priority/) — Priority is a number on a configured scale, set by hand or scored by a framework; difficulty is its twin.
- [Configuration](https://mlavrinenko.github.io/mindtape/config/) — How .mindtape/config.toml overlays a defaults floor through a four-layer merge, and the sections it can set.
- [Canonical schema](https://mlavrinenko.github.io/mindtape/schema/) — The task-JSON schema mt ls --json and mt import share — the contract a jq adapter codes against.
- [Import & export](https://mlavrinenko.github.io/mindtape/import-export/) — Bring tasks in from another tracker with mt import and jq, and reshape them back out with the same tools.
- [For AI agents](https://mlavrinenko.github.io/mindtape/agents/) — What makes MindTape legible to a coding agent: structured JSON, describable schemas, honest exit codes, and file-based state.
- [Internals](https://mlavrinenko.github.io/mindtape/internals/) — How MindTape is built: the crate layout, the content-hash cache that keeps live evaluation cheap, and what differs per platform.
- [Comparison](https://mlavrinenko.github.io/mindtape/comparison/) — How MindTape compares, feature by feature, to the other local file-and-CLI task trackers, and when one of them fits better.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).
