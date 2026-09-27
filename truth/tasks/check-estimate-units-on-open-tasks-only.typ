#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Check estimate units on open tasks only",
  kind: "feat",
  priority: ice(
    impact: 5,
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
      enforced-practice: helps[the unit list governs the work still to do, not finished history],
      cheap-adoption: helps[a project renames or retires a unit with one edit to `units`],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 9, 27),
)

== Problem

+ A project declares `[tasks.estimate] units = ["files"]`; 362 closed tasks carry `estimate: (files: N)`.
+ It moves to `units = ["src-files"]`.
+ 🔴 `mt check` fails every closed task with `` `files` is not an allowed estimate unit ``: `check_estimate` in `crates/mindtape/src/validate.rs` checks the unit list before its terminal exemption, which only the bounds get. Retiring a unit takes keeping it in `units` plus a rule forbidding it on open tasks.


== TO-BE

- 🟢 `units` is checked on open tasks only, like the bounds: a closed task keeps whatever unit it was estimated in.
- 🟢 Retiring a unit is removing it from `units`; an open task still carrying it is refused at `mt add`, `mt set` and `mt check`.


== Changes

- `crates/mindtape/src/validate_tests_fields.rs` (red first): a closed task with a retired unit is clean; an open one is refused.
- `crates/mindtape/src/validate.rs`: move the unit check below the terminal return.
- `www/content/estimates.typ`: say `units` governs open tasks.
- `CHANGELOG.md`: the change.


== Alternatives

- A legacy rule per retired unit (`miss(estimate.files)` on open tasks): two edits per retirement, and the unit stays legal everywhere.
- Replace `units` with a rule (`only(estimate, "src-files")`): a new predicate, and rules lose the typo check on `estimate.<unit>`.


== Out of scope

- Tags: `truth/tasks/check-allowed-tags-on-open-tasks-only.typ`.


== Dogfood

`truth/tasks/move-open-tasks-and-templates-to-a-src-files-estimate.typ` retires `files` here with a one-line `units` edit.

