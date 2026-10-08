#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Add --json to mt graph and mt preset ls and require it on every read command",
  kind: "dev",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 2,
  estimate: (src-files: 4),
  tags: ("found", "cli"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[a new read command without --json fails the CLI tree test],
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[agents.typ's every-read-command-prints-JSON claim stays true],
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 8),
  links: depends-on("print-mt-checks-problems-as-json-with-json.typ"),
)

== Problem

+ Reviewing the docs before release, checking agents.typ against each read command's flags
+ 🔴 #raw("crates/mindtape-cli/src/cli/tree_json.rs"): mt graph and mt preset ls have no --json, and nothing fails when a read command lacks it

```
graph: format   preset ls: (none)
```
(from `mt _cli`; docs/contributing/command-semantics.md classes both as read commands)


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape-cli/src/cli/tree_json.rs").

== TO-BE

- 🟢 `mt graph --json` prints the nodes and typed edges it would draw, and `mt preset ls --json` prints each preset's name, origin and description.
- 🟢 A test fails, naming the command, when a visible command whose class in `cli/command_class.rs` is read has no `--json` flag. `config schema`, whose output already is JSON, is the one named exemption.


== Changes

- `crates/mindtape-cli/src/cmd/graph/`: `--json` beside `--format`.
- `crates/mindtape-cli/src/cmd/preset.rs`: `--json` on `ls`.
- `crates/mindtape-cli/src/cli/tree_json.rs` tests: walk the tree, refuse a read command without `--json`.
- `docs/src/contributing/command-semantics.typ`: say a read command takes `--json` and the test enforces it; run `just docs md`.
- `CHANGELOG.md`: an `[Unreleased]` entry.


== Rollback

- Revert the commit.


== Out of scope

- `--json` on write commands.


== Dogfood

The next read command added here fails `just test` until it prints JSON.

