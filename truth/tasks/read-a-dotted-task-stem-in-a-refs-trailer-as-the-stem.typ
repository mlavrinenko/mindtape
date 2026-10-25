#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Read a dotted task stem in a Refs trailer as the stem",
  kind: "fix",
  priority: ice(
    impact: 7,
    confidence: 0.8,
    ease: 8,
  ),
  difficulty: 2,
  tags: ("mt",),
  estimate: (src-files: 2),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: helps[a `Refs:` trailer names the task it works on, whatever its stem holds],
      enforced-practice: helps[`mt check --commits` stops refusing a trailer that names a real task],
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
  status: done(2026, 10, 25),
)

== Problem

+ In a project whose task stems hold a dot, such as speconaut's `foo.task-261023.typ`, commit with the trailer `Refs: foo.task-261023`.
+ Run `mt check --commits HEAD~1..HEAD`.
+ 🔴 It prints ``error: <sha>: Refs: `foo.task-261023` names no task``.
+ Run `mt ls 'touches("<a file that commit changed>")'`.
+ 🔴 It prints `0 of N tasks`.
`value_stem` in `crates/mindtape/src/commit/mod.rs` runs `Path::file_stem` on every trailer value, so `foo.task-261023` reads as `foo`.


== Environment

- `mt 0.3.0`, on Linux, against speconaut at `2e0b8fdd`, where commit `26a22e25` reproduces both steps.


== TO-BE

- 🟢 A bare trailer value is the stem as written; `Refs: foo.task-261023` names `foo.task-261023.typ`.
- 🟢 A value holding `/` or `\` or ending in `.typ` still reads as its file stem, and `ref_problem` still asks for the bare stem.


== Changes

- `crates/mindtape/src/commit/tests.rs`: the red test first, `value_stem("foo.task-261023")` returning `foo.task-261023`, plus a `touches()` case over a dotted stem.
- `crates/mindtape/src/commit/mod.rs`: in `value_stem`, take `file_stem` only when the value holds `/` or `\` or ends in `.typ`, the test `ref_problem` already makes.


== Out of scope

- The path-form warning in `ref_problem`, which already reads right.


== Dogfood

Not applicable: this repo's stems hold no dot. speconaut, which names tasks `<slug>.task-<date>`, is the user.

