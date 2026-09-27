#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Count changed files per git state in filters",
  kind: "feat",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 3,
  tags: ("mt",),
  estimate: (files: 5),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[a project writes its own budget over any unit and any git state],
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: helps[git is read on demand by the expression that asks, no index kept],
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 9, 27),
  links: related(
    "evaluate-task-scope-commit-rules.typ",
  )[both read HEAD, index and working tree; share one git state reader],
)

== Problem

+ A project wants a warning when the work on a task outgrows the source files it planned, counting source only.
+ 🔴 It cannot: the only git-aware check is `crates/mindtape/src/estimate_drift.rs`, hardwired to the `files` unit and the unstaged-plus-staged tree, with `[tasks.estimate.drift] exclude` as its one knob. It counts docs, tests and `CHANGELOG.md`, which every task here touches by process, so it warned on every task in one session and was ignored.


== TO-BE

- 🟢 `changed(<states>, <glob>…)` counts distinct changed paths. `<states>` is `staged`, `worktree` (unstaged and untracked) or both joined with `+`. The globs name what counts; one starting with `!` excludes; no glob means every path.

```toml
[[tasks.rules]]
name     = "src-budget"
when     = 'status == "wip" && has(estimate.src-files)'
assert   = 'changed("staged+worktree", "crates/*/src/**", "!**/*_tests*.rs") <= estimate.src-files'
severity = "warn"
```

- 🟢 Git runs read-only (`--no-optional-locks`) and at most once per state per command.
- 🟢 Outside a git repository a rule reading `changed()` is skipped and reported once as skipped, never an error; a filter typed on the command line fails naming the reason.


== Changes

- `crates/mindtape/src/git_state_tests.rs` (red first): porcelain parsing per state, rename destinations, include and `!` globs.
- `crates/mindtape/src/git_state.rs` (new): the git reader, moved out of `estimate_drift.rs` rather than copied.
- `crates/mindtape/src/filter_predicates.rs`: `changed()`.
- `www/content/filters.typ`: document it and the state names.
- `CHANGELOG.md`: the addition.


== Alternatives

- `lines()` for a per-file line budget: dropped. The drift check's 200-lines-per-file ceiling has no record of catching anything; add it when a rule needs it.
- One hardcoded working-tree state: what the drift check does today; `staged` is what a commit-time rule reads.
- A `committed` state attributed by `Refs:` trailers: dropped with `truth/tasks/add-the-committed-state-to-changed-and-lines.typ`. Work here lands in one commit per task, so the worktree already sees all of it.


== Out of scope

- Retiring the drift check: `truth/tasks/retire-the-drift-check-and-estimate-bounds-in-favour-of-rules.typ`.


== Dogfood

This repo's source budget becomes the rule above in the retirement task.
