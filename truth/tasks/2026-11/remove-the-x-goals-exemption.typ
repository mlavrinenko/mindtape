#import "/truth/prelude/2/lib.typ": *

#show: task.with(
  title: "Remove the x-goals exemption from the live goal rules",
  kind: "dev",
  priority: ice(
    impact: 7,
    confidence: 0.9,
    ease: 5,
  ),
  difficulty: 3,
  links: (
    (
      task: "rewire-goals-through-typst-markers.typ",
      kind: "surfaced-by",
      note: none,
    ),
  ),
  tags: ("found", "mt", "gating", "stateless", "no-ui-ux"),
  assignee: "agent",
  status: proposed(2026, 11, 22),
)

== AS-IS

+ Reviewing the goal DSL milestone after it landed.
+ 🔴 #raw(".mindtape/config.toml"): any open task carrying `x-goals` skips the goal check and only warns, so the field is a permanent bypass.

The live rule has `when = "!terminal && !has(x-goals)"`, the warning is `severity = "warn"`, and `check_goals_e2e.rs::a_reopened_legacy_task_passes_with_a_warning` codifies the pass.

== TO-BE

- 🟢 Every open task gets the goal check. An open task with `x-goals` fails until its goals move into the new DSL.
- 🟢 The warning-only rule and the old-name aliases disappear. A reopened legacy task moves to the new DSL instead of passing.
- 🟢 Closed history may keep `x-goals` as data. A task that opens again must carry the new form.

```toml
[[tasks.rules]]
name = "give-every-goal-verdict"
when = "!terminal"
check = { typst = "truth/lib.typ", function = "check-goal-over", field = "markers" }
```


== Changes

- `.mindtape/config.toml`: drop `!has(x-goals)` from the goal, content and unique rules, and delete `warn-on-x-goals-on-open-tasks`.
- `truth/goal-calls.typ`: drop the old x-goals name aliases and keep the names the new DSL writes.
- `crates/mindtape-cli/tests/it/check_goals_e2e.rs`, `goal_markers_e2e.rs`: replace the pass-with-warning case with a refusal, and migrate any open x-goals fixture.

== Out of scope

- Rewriting closed history. The field stays as a record, and only reopened tasks pay the migration.

== Dogfood

A reopened old task gets the same goal check as a new one, so the exemption cannot hide a missing verdict.

== Goals

+ #Helps(adopter, to: run-checks-at-the-write)[no open task opts out of the goal check].
+ #Neutral(read-tasks-at-a-glance, guide-the-implementer, keep-the-engine-agnostic, adopt-in-one-line, beat-markdown, keep-plain-files, explain-itself, keep-changes-safe, match-docs-to-code).

