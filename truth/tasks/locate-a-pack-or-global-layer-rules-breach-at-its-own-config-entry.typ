#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Locate a pack or global-layer rule's breach at its own config entry",
  kind: "feat",
  priority: ice(
    impact: 4,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 3,
  tags: ("mt",),
  estimate: (src-files: 5),
  x-goals: (
    product: (
      readable-tasks: helps[a skipped rule opens at the entry that is at fault],
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: helps[one origin record answers where a rule is declared, in place of a second line scan],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 5),
)

== Problem

+ Run `mt check` in a project whose `extends = ["mindtape-tasks"]` pack has a `[[tasks.rules]]` rule that cannot be evaluated, for example one whose `changed()` git cannot answer.
+ 🔴 The warning `skipped: …` points at the first task's `task.with(…)` line. `config_entry` in `crates/mindtape/src/config_rules_eval.rs` scans only the project's `.mindtape/config.toml`, so a rule from a pack or a global config layer has no location.


== TO-BE

- 🟢 Config loading records each rule's origin with the line of its table header: a project or global layer as `file:line`, a path pack as its file, a built-in pack as `pack <name>:<line>`.
- 🟢 A skipped rule points there, whichever layer declared it.
- 🟢 `mt config show --origin` and the location read one record, the `RuleOriginEntry` list `config_packs.rs` already builds.
- 🟢 The line scan `config_entry` is gone.


== Changes

- `crates/mindtape/src/config_rules_eval_tests.rs`: the red test first, a pack rule skipped and pointing at its `pack <name>:<line>`.
- `crates/mindtape/src/violation.rs`: a `Location::Pack` variant, shown as `pack <name>:<line>`.
- `crates/mindtape/src/config_packs.rs`: read each source's `[[…rules]]` header lines in order and store the line on each `RuleOriginEntry`.
- `crates/mindtape/src/config_layers.rs`, `crates/mindtape/src/config.rs`: keep the entries on the loaded `Config`, a derived field outside the schema.
- `crates/mindtape/src/config_rules_eval.rs`: read the location from the config, and delete the line scan.


== Alternatives

- Extend the line scan to the pack and global files: a second parse of each file that can disagree with the merge about which entry won.
- Carry TOML spans through the merge: the `toml` table drops them, and the merge works on plain values.
- Leave the marker line: the reader opens the wrong file for a fault in the config.


== Out of scope

- A breach of a rule's `assert`, which belongs to the task and keeps the task's marker line.
- Rules written as an inline array rather than `[[…]]` tables, which keep the marker line.


== Dogfood

This repo extends `mindtape-tasks`, so a skipped pack rule points at `pack mindtape-tasks:<line>`.
