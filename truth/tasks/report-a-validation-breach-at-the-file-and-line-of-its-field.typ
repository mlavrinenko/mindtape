#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Report a validation breach at the file and line of its field",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 3,
  estimate: (src-files: 2),
  tags: ("found", "mt"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[a reader of `mt check` finds every breach by file and line],
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
  status: done(2026, 10, 3),
)

== Problem

+ Reviewing the uniform breach shape of \`mt check\` after it landed
+ 🔴 #raw("crates/mindtape-cli/src/check.rs"): A violation from validate.rs carries no location, so format\_violation prints the task stem where every other breach prints a file and line.

+ Copy `examples/minimal` two directories below a copy of `lib/`, so its prelude path resolves.
+ Add `links: depends-on("nope.typ"),` to `tasks/setup-ci.typ`.
+ Run `mt check` there.
+ 🔴 It prints `error: setup-ci: depends-on: link target …`, with the stem in place of `tasks/setup-ci.typ:<line>`.


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape/src/validate.rs").

== TO-BE

- 🟢 Every breach from `crates/mindtape/src/validate.rs` carries a location: the line of the offending field, else the task's `task.with(` line.
- 🟢 `format_violation` in `crates/mindtape-cli/src/check.rs` loses its stem-only branch.


== Changes

- `crates/mindtape/src/validate.rs`: attach a location to each violation, the field's line where the file has it, else the `task.with(` line `gate.rs` already falls back to.
- `crates/mindtape-cli/src/check.rs`: drop the stem-only branch; a red test pins a link breach printing `file:line`.


== Rollback

Revert the commit; validation breaches print the stem again.

== Out of scope

- Load errors, which already print the file.


== Dogfood

This repo's `mt check` prints one breach shape, so an editor jumps to every breach.

