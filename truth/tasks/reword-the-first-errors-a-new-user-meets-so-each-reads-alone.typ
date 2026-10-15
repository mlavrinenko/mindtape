#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Reword the first errors a new user meets so each reads alone",
  kind: "fix",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 2,
  tags: ("cli",),
  estimate: (src-files: 10),
  x-goals: (
    product: (
      readable-tasks: helps[a refusal says what to do next in words that need no other line],
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[a new user's first refusals explain themselves],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 15),
)

== Problem

+ Run `mt init` in an empty git repo, then `mt add Fix the login`.
+ Run `mt done fix-the-login`.
+ 🔴 It prints `hint: or pass --force to override`, a hint that opens with `or` and nothing before it.
+ Run `mt ls 'status == wip'`.
+ 🔴 It prints `error: unknown field: 'wip' in filter 'status == wip'` and no hint that `wip` wants quotes.
+ In a project whose template requires slots, run `mt add "Log slow queries" --print`.
+ 🔴 It prints `error: missing required slots: 'to-be, changes, out-of-scope'`, three names in one pair of quotes.
+ Add a task a rule refuses.
+ 🔴 It prints `error: the task as filled:2: rules.x: ...`, a place that reads like a broken file path.


== Environment

- `mt` built from `main` at the commit that filed this, on Linux.


== TO-BE

- 🟢 A refused flip says `hint: flip to an allowed status first, or pass --force to skip this check`, and the other guards word their own hint the same way.
- 🟢 `mt ls 'status == wip'` adds `hint: to match text, quote it, as in status == "wip"`, built from the user's own expression.
- 🟢 `error: missing required slots: 'to-be', 'changes', 'out-of-scope'`.
- 🟢 A breach in a task not yet written reads `<new task from template 'task'>:2`, or `<new task>:2` when no template made it.


== Changes

- `crates/mindtape-cli/src/diag/tests_fill.rs`, `diag/from_lib_tests.rs`, `cmd/flip/tests*.rs`: the red tests first.
- `crates/mindtape-cli/src/diag/mod.rs`, `diag/render.rs`: `Diag::values`, each value quoted.
- `crates/mindtape-cli/src/diag/hint.rs`: a hint that ends with an example built from the input.
- `crates/mindtape-cli/src/diag/from_lib.rs`: the slot list, and the quoting hint for an unknown name compared as a value.
- `crates/mindtape-cli/src/diag/quoting.rs`: the filter rebuilt with that name quoted, outside strings and only where a field is on the other side.
- `crates/mindtape/src/validate/violation.rs`: `Location::Filled` names its template when one is known.
- `crates/mindtape-cli/src/task_edit.rs`, `cmd/add/mod.rs`, `cmd/add/hint.rs`, `pre_write/mod.rs`: pass the template name through.
- `crates/mindtape-cli/src/cmd/flip/guards.rs`: a hint per guard that stands alone.


== Out of scope

- The order of `hint:` and `note:` lines. Every message prints hints first, and one message out of that order would read worse than the order itself.
- `mt flip`'s gate hints in `cmd/flip/hooks.rs`. Their `or` follows a first hint, so it reads.


== Dogfood

This repo's `mt add` and `mt done` refusals print the new wording.

