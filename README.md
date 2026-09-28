<!-- Generated from docs/src/readme.typ by `just docs md`. Do not edit; edit the source. -->

<h1 align="center"><img src="https://raw.githubusercontent.com/mlavrinenko/mindtape/main/www/assets/images/logo.svg" alt="MindTape" width="120" /><br />
 MindTape</h1>

<h4 align="center">A <a href="LICENSE">MIT</a>-licensed, daemon-free file-based task tracker powered by <a href="https://typst.app/">Typst</a>.</h4>

<h4 align="center"><a href="https://mlavrinenko.github.io/mindtape/">Documentation</a> · <a href="https://mlavrinenko.github.io/mindtape/quickstart/">Quickstart</a> · <a href="https://mlavrinenko.github.io/mindtape/comparison/">Comparison</a></h4>

---

MindTape keeps no daemon, database or index: every command evaluates your task files where they sit, so the repository is the tracker. A task is one Typst file, its `id` is the filename stem, and links target files — rename one without updating its dependents and `mt check` says so instead of dangling.

Say the same task lived as a Markdown file with YAML frontmatter instead, checked by a script. Here is one real, closed task from this repo’s own backlog, both ways.

```yaml
---
title: Move goal verdicts into x-goals
kind: dev
# a script keeps these three numbers in bounds and combines them into a
# score itself; nothing here says impact tops out at 10
priority: {impact: 7, confidence: 0.9, ease: 6}
difficulty: 2
tags: [mt]
estimate: {files: 15}
assignee: agent
status: done
status_date: "2026-09-16"
status_note: >
  verdicts landed in x-goals across every open task and template;
  check-goals is the only place they're checked now
# a script resolves this filename by hand to know whether the task is
# ready or blocked
depends_on: [check-a-task-field-with-a-typst-function.md]
# a script checks every key below is present, every effect is one of
# helps/hurts/neutral, and every helps/hurts carries a reason
x_goals:
  product:
    readable-tasks:
      effect: helps
      reason: the body runs from Problem straight to TO-BE, and the verdicts sit beside status and links
    implementable-tasks:
      effect: neutral
    enforced-practice:
      effect: helps
      reason: a skipped goal becomes a named rules.goals error that mt add refuses, not an eval panic
    cheap-adoption:
      effect: neutral
    typst-over-markdown:
      effect: neutral
    plain-files:
      effect: neutral
  process:
    sound-codebase:
      effect: neutral
    docs-match-code:
      effect: neutral
---
```

Nothing above is checked until a script reads the file and does it by hand: parses `status_date`, keeps `priority`’s three numbers in bounds, resolves `depends_on` against another file on disk, and confirms every `x_goals` key is present with a reason.

The same task as Typst — `truth/tasks/move-goal-verdicts-into-x-goals.typ` (trimmed here: the file continues with `== Problem`, `== TO-BE`, `== Changes`, `== Rollback`, `== Out of scope` and `== Dogfood`):

```typ
#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Move goal verdicts into x-goals",
  kind: "dev",
  priority: ice(
    impact: 7,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 2,
  tags: ("mt",),
  estimate: (files: 15),
  assignee: "agent",
  status: done(
    2026,
    9,
    16,
  )[verdicts landed in x-goals across every open task and template; check-goals is the only place they're checked now],
  links: depends-on("check-a-task-field-with-a-typst-function.typ"),
  x-goals: (
    product: (
      readable-tasks: helps[the body runs from Problem straight to TO-BE, and the verdicts sit beside status and links],
      implementable-tasks: neutral,
      enforced-practice: helps[a skipped goal becomes a named `rules.goals` error that `mt add` refuses, not an eval panic],
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
)
```

What only the Typst side does:

- **Typed values with their own shape.** `status: wip(2026, 9, 14)[note]`, `priority: ice(impact: 8, confidence: 0.7, ease: 6)` — validated on every command, never parsed as strings.
- **Typed links.** `depends-on("other.typ")` — resolved and checked on every command, feeding `ready`/`blocked` and the graph; a renamed target fails `mt check` instead of dangling.
- **A repo-invented field.** Here `x-goals`, checked by the repo’s own Typst function named in config, and refused before `mt add` writes:

```toml
[[tasks.rules]]
check = { typst = "truth/lib.typ", function = "check-goals", field = "x-goals" }
```

The full model is on [Concepts](https://mlavrinenko.github.io/mindtape/concepts/). Every field a task file can declare is on [Task DSL](https://mlavrinenko.github.io/mindtape/dsl/).

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
- [Comparison](https://mlavrinenko.github.io/mindtape/comparison/) — How MindTape compares, feature by feature, to the other local CLI task trackers, agent-era ones included, and when one of them fits better.
- [Concepts](https://mlavrinenko.github.io/mindtape/concepts/) — The model behind MindTape: files as tasks, live evaluation, identity by filename, one extension seam, and the two layers that enforce a task.
- [Task DSL](https://mlavrinenko.github.io/mindtape/dsl/) — Every field a task file can declare, the status shortcuts, and what the body can hold.
- [Task links](https://mlavrinenko.github.io/mindtape/links/) — Typed edges between tasks: the built-in kinds, project-declared kinds with their own target validation, and what mt check refuses.
- [Recurring tasks](https://mlavrinenko.github.io/mindtape/recurring/) — One task that repeats: every() for the common cadences, rrule() for calendar rules, and the command that advances the series.
- [Templates & slots](https://mlavrinenko.github.io/mindtape/templates/) — How mt add scaffolds a task from a Typst template whose values are slot() calls.
- [Derived tags](https://mlavrinenko.github.io/mindtape/derived-tags/) — Tasks inherit tags from their location under a root, without those tags ever being written to the file.
- [CLI reference](https://mlavrinenko.github.io/mindtape/cli/) — Every mt command, grouped, with generated transcripts and pointers to the pages that own each deep topic.
- [Filter language](https://mlavrinenko.github.io/mindtape/filters/) — The expression language mt ls and mt rm use to select tasks: fields, operators, functions, and dependency predicates.
- [Priority](https://mlavrinenko.github.io/mindtape/priority/) — Priority is a number on a configured scale, set by hand or scored by a framework; difficulty is its twin.
- [Configuration](https://mlavrinenko.github.io/mindtape/config/) — The .mindtape/config.toml reference: the four-layer merge, rule packs, the section map, and the keys only config defines.
- [Rule packs](https://mlavrinenko.github.io/mindtape/rule-packs/) — A rule pack is a project's task practice as one importable unit: extends adopts it, a same-named rule overrides one entry, disable drops one. What a pack file may contain, how it resolves, the merge order, and what mindtape-tasks enforces.
- [Commit rules](https://mlavrinenko.github.io/mindtape/commit-rules/) — How a project holds commits to its rules: the \`Refs\` trailer \`mt check --commit-msg\` reads, commit-scope filter expressions, and re-checking a pushed range in CI.
- [Named views and dictionaries](https://mlavrinenko.github.io/mindtape/named-sets/) — Two top-level config sections that name something once so every rule can point at it: a view bundles a filter with a sort and columns, a dictionary names a set of strings.
- [Flip hooks](https://mlavrinenko.github.io/mindtape/hooks/) — How a project binds actions to a task changing status: printing a message, listing what else is in flight, or running a command that may refuse the flip.
- [Estimate caps and working-tree budgets](https://mlavrinenko.github.io/mindtape/estimates/) — How tasks.estimate declares the legal units, and how plain rules cap a declared estimate and hold the git working tree to it.
- [Canonical schema](https://mlavrinenko.github.io/mindtape/schema/) — The task-JSON schema mt ls --json and mt import share — the contract a jq adapter codes against.
- [Import & export](https://mlavrinenko.github.io/mindtape/import-export/) — Bring tasks in from another tracker with mt import and jq, and reshape them back out with the same tools.
- [Self-validating tasks](https://mlavrinenko.github.io/mindtape/self-validating-tasks/) — A repo's tasks can carry a structure it invented, checked by the repo's own Typst function on every write — no separate linter to maintain.
- [For AI agents](https://mlavrinenko.github.io/mindtape/agents/) — What makes MindTape legible to a coding agent: structured JSON, describable schemas, honest exit codes, and file-based state.
- [Internals](https://mlavrinenko.github.io/mindtape/internals/) — How MindTape is built: the crate layout, the content-hash cache that keeps live evaluation cheap, and what differs per platform.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).
