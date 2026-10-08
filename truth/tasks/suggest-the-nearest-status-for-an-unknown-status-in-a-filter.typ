#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Suggest the nearest status for an unknown status in a filter",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 1,
  estimate: (src-files: 2),
  tags: ("found", "cli"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[filters.typ's wpi-suggests-wip example becomes true],
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 8),
)

== Problem

+ Reviewing the docs before release, running the example from filters.typ:177
+ 🔴 #raw("crates/mindtape/src/filter/mod.rs"): an unknown status in a filter names the value but suggests nothing, while filters.typ promises a suggestion and unknown fields and dicts already get a nearest-name hint

```
views.x.filter: unknown status in filter: 'wpi'
```
`crates/mindtape/src/filter/mod.rs:145`: `#[error("unknown status in filter: '{value}'")]`, with no suggestion field.


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape/src/filter/mod.rs").

== TO-BE

- 🟢 `status == "wpi"` in a project whose legend has `wip` fails with `hint: did you mean wip?` and the legend's statuses as choices, the way an unknown field reads.


== Changes

- `crates/mindtape/src/filter/mod.rs`: carry the nearest status and the choices on the error, as the other nearest-name errors do.
- `crates/mindtape-cli/src/diag/from_lib.rs`: render them as `hint:` lines.
- `CHANGELOG.md`: an `[Unreleased]` entry.


== Rollback

- Revert the commit.


== Out of scope

- Other unknown-value errors that already suggest.


== Dogfood

A typo in a view filter in this repo's config names the status meant.

