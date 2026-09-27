#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Count changed files and lines per git state in filters",
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

+ A project wants a warning when staged work exceeds a task's budget, counting only `src/**`.
+ 🔴 It cannot: the only git-aware check is `crates/mindtape/src/estimate_drift.rs`, which reads the unstaged-plus-staged tree, against the `files` unit only, with `[tasks.estimate.drift] exclude` as its one knob.


== TO-BE

- 🟢 `changed(<states>, <glob>…)` counts distinct changed paths; `lines(<states>, <glob>…)` sums added plus deleted lines. `<states>` is `staged`, `worktree` (unstaged and untracked) or both joined with `+`. A glob starting with `!` excludes; no glob means every path.

```toml
[[tasks.rules]]
name     = "file-budget"
when     = 'status == "wip" && has(estimate.files)'
assert   = 'changed("staged+worktree", "!www/generated/**") <= sum(estimate.files, status == "wip")'
severity = "warn"
```

- 🟢 Git runs read-only (`--no-optional-locks`) and at most once per state per command.
- 🟢 Outside a git repository a rule reading `changed()` or `lines()` is skipped and reported once as skipped, never an error; a filter typed on the command line fails naming the reason.


== Changes

- `crates/mindtape/src/git_state_tests.rs` (red first): porcelain and numstat parsing per state, rename destinations, globs with `!`.
- `crates/mindtape/src/git_state.rs` (new): the git reader, moved out of `estimate_drift.rs` rather than copied.
- `crates/mindtape/src/filter_predicates.rs`: `changed()` and `lines()`.
- `www/content/filters.typ`: document both and the state names.
- `CHANGELOG.md`: the additions.


== Alternatives

- Per-state wrappers (`staged(changed())`) as `truth/tasks/evaluate-task-scope-commit-rules.typ` has for task fields: a changed-path count has no per-task value to wrap; the state is the argument.
- One hardcoded "working tree" state: what the drift check does today, and why a staged-only budget cannot be written.


== Out of scope

- The `committed` state: `truth/tasks/add-the-committed-state-to-changed-and-lines.typ`.
- Retiring the drift check: `truth/tasks/retire-the-drift-check-and-estimate-bounds-in-favour-of-rules.typ`.


== Dogfood

This repo's file budget becomes the rule above in the retirement task.

