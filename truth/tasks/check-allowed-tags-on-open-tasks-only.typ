#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Check allowed tags on open tasks only",
  kind: "feat",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 9,
  ),
  difficulty: 1,
  tags: ("mt",),
  estimate: (files: 1),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[the tag list governs the work still to do, not finished history],
      cheap-adoption: helps[a project renames or drops a tag without touching finished tasks],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: wip(2026, 9, 27),
)

== Problem

+ A project drops `speconaut` from `[tasks.tags] allowed`.
+ 🔴 `mt check` fails every closed task tagged `speconaut`: `check_tags` in `crates/mindtape/src/validate.rs` checks authored and folder-derived tags on every task, so dropping a tag means rewriting finished history or keeping the tag legal forever.


== TO-BE

- 🟢 `allowed` is checked on open tasks only, authored and derived tags alike: a closed task keeps the tags it was filed with.
- 🟢 Dropping a tag is removing it from `allowed`; an open task still carrying it is refused at `mt add`, `mt set` and `mt check`.


== Changes

- `crates/mindtape/src/validate_tests_fields.rs` (red first): a closed task with a dropped tag, authored and derived, is clean; an open one is refused.
- `crates/mindtape/src/validate.rs`: skip `check_tags` for terminal tasks.
- `www/content/config.typ`: say `allowed` governs open tasks.
- `CHANGELOG.md`: the change.


== Alternatives

- Keep dropped tags in `allowed` with a comment: the list stops saying what a new task may use.


== Out of scope

- Assignees and enum fields such as `kind`: the same question, answered when a project retires one.
- Status values: a closed task is recognised as closed through the legend, so the legend must keep its values.


== Dogfood

This repo's tag list can drop a retired area tag with one edit.

