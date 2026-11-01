#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Fold repeated Typst warnings in mt check into one line per kind",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 3,
  estimate: (src-files: 2),
  tags: ("found", "cli"),
  x-goals: (
    product: (
      readable-tasks: helps[a check whose real problems are not buried under hundreds of warning lines],
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 11, 1),
)

== AS-IS

+ Migrating kimmable's config to mt 0.3.0
+ 🔴 #raw("crates/mindtape-cli/src/cmd/check/mod.rs"), #raw("crates/mindtape-cli/src/util.rs"): `mt check` and `mt ls` print each Typst warning in full with its hint, so one habit repeated across a backlog floods the output

In kimmable, `mt check 2>&1 | rg -c 'no text within stars'` prints `188`, each a warning line plus the same hint:
```
warning: tasks/a-contrib-driver-for-the-frame-at-a-timestamp.typ:43:41: no text within stars
hint: using multiple consecutive stars (e.g. **) has no additional effect
```

This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in the shared load
report, so every command that prints a Typst warning folds it.

== TO-BE

- 🟢 `mt check` and `mt ls` print a repeated warning once, with its hint, the count and the files, and `--verbose` still lists each site.

== Changes

- `crates/mindtape-cli/src/warnings.rs` (new): fold warnings with the same message into one line naming the hint, the count and the files; `fold_warnings` keeps every site when asked.
- `crates/mindtape-cli/src/util.rs`: `load_problems` folds for every command that reports load warnings, so `mt ls`, `mt find` and a set share the behavior; `folded_load_problems` passes the choice explicitly.
- `crates/mindtape-cli/src/cmd/check/mod.rs`, `cmd/check/set.rs`, `project_set.rs`: keep every site for `--json` and `-v`.
- `crates/mindtape-cli/src/main.rs`, `cli/mod.rs`: carry the `-v` flag to the report and say in its help that it lists every site.
- Tests in `warnings_tests.rs`, `cmd/check/tests.rs`, `tests/it/breach_location_e2e.rs`; the changelog, the `check` and `ls` docs pages and the outdatty lock.

== Rollback

- Revert the grouping and print every warning again.

== Out of scope

- kimmable's task bodies, which write markdown `**bold**` in Typst. Rewriting them is that repo's own task.

== Dogfood

This repo's `mt check` keeps its one-line success, `mt ls` stays readable on a backlog with a repeated warning, and the shared report folds it.
