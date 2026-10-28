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
  status: proposed(2026, 10, 28),
)

== Problem

+ Migrating kimmable's config to mt 0.3.0
+ 🔴 #raw("crates/mindtape-cli/src/cmd/check/mod.rs"): `mt check` prints each Typst warning in full with its hint, so one habit repeated across a backlog floods the output

In kimmable, `mt check 2>&1 | rg -c 'no text within stars'` prints `188`, each a warning line plus the same hint:
```
warning: tasks/a-contrib-driver-for-the-frame-at-a-timestamp.typ:43:41: no text within stars
hint: using multiple consecutive stars (e.g. **) has no additional effect
```

This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape-cli/src/cmd/check/mod.rs").

== TO-BE

- 🟢 `mt check` prints a repeated warning once, with its hint, the count and the files, and `--verbose` still lists each site.

== Changes

- `crates/mindtape-cli/src/cmd/check/mod.rs`: group warnings by message, and print each group once with its hint, count and files.

== Rollback

- Revert the grouping and print every warning again.

== Out of scope

- kimmable's task bodies, which write markdown `**bold**` in Typst. Rewriting them is that repo's own task.

== Dogfood

This repo's `mt check` keeps its one-line success, and a backlog with a repeated warning stays readable.
