#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Add nonminimal_bool and redundant_guards to the workspace lint set",
  kind: "dev",
  priority: ice(
    impact: 3,
    confidence: 0.9,
    ease: 9,
  ),
  difficulty: 1,
  tags: ("tooling", "tests"),
  estimate: (src-files: 1),
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
      sound-codebase: helps[a lint fails before a regression reaches review],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: cancelled(
    2026,
    10,
    1,
  )[No-op: a workspace deny is as overridable by a module \#\[allow\] as -D warnings already is, and the cited looks\_like\_path clause is a MAIN\_SEPARATOR comparison neither lint flags.],
)

== Problem

+ Read `Cargo.toml`'s `[workspace.lints.clippy]` (line 30).
+ 🔴 `if_same_then_else` (line 47) and `match_same_arms` (line 134) are explicit `deny`s, but `nonminimal_bool` and `redundant_guards` are absent: they only gate through the clippy recipe's `-D warnings` (`Justfile:86`), so a module-level `allow` or a recipe change drops them with nothing to notice.
+ The 2026-09-30 sweep's hand-excluded equivalents (`A || A`-style clauses, duplicate guards) are the class the pair targets.

== TO-BE

- 🟢 `Cargo.toml` lists `nonminimal_bool` and `redundant_guards` beside the other denies, each with the one-line reason the pair earns its keep.
- 🟢 `just clippy` and `just check` stay green.

== Changes

- `Cargo.toml`: add `nonminimal_bool = "deny"` and `redundant_guards = "deny"` under Code Quality.
- Fix any hit the explicit deny surfaces (none expected: `-D warnings` already fails them today).

== Rollback

Remove the two lines; the lints return to their warn-by-default state under `-D warnings`.

== Out of scope

- `clippy::pedantic`/`nursery` wholesale: separate decision, mostly noise.
- A custom ast-grep pass: that is the tautology task, not this one.

== Dogfood

A future `A || A` fails at the lint, before it can become an equivalent mutant and a triage cycle.
