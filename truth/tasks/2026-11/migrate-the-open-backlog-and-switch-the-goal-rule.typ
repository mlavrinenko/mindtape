#import "/truth/prelude/2/lib.typ": *

#show: task.with(
  title: "Migrate the open backlog and switch the goal rule",
  kind: "dev",
  priority: ice(
    impact: 7,
    confidence: 0.9,
    ease: 4,
  ),
  difficulty: 4,
  tags: ("mt", "docs", "tooling", "stateless"),
  assignee: "agent",
  status: done(2026, 11, 22),
  links: parent("rewire-goals-through-typst-markers.typ")
    + depends-on(
      "define-the-goal-dsl-over-metadata-markers.typ",
      "carry-body-markers-through-the-candidate-door.typ",
      "emit-project-imports-when-a-writer-rebuilds-a-task-file.typ",
      "decide-the-goal-set-for-an-agnostic-engine.typ",
    ),
)

== AS-IS

+ 🔴 Open any proposed task. Its verdicts are header dicts of strings. A maintainer-only gate can claim an adopter goal.
+ Open a closed task. 🔴 It has the same `x-goals` field, and a blanket migration would rewrite history for no check.
+ Read `.mindtape/config.toml`. 🔴 `give-every-goal-verdict` reads `x-goals` through `check-goals`. The body markers the DSL emits reach no rule.

== TO-BE

- 🟢 The registry files carry the chosen names. `truth/lib.typ` keeps `check-goal-over` and `surfaced-by`.
- 🟢 The live rule reads `markers`, allows `Goals` after `Dogfood`, caps it at 160 words of text and warns on `x-goals` with a message. `emit-imports` lists generation 2.
- 🟢 Every open task ends with a rated `Goals` block and no `x-goals`. Closed tasks keep `x-goals`.
- 🟢 Tasks add `/truth/prelude/2/lib.typ`. The emitter writes it on `mt import`.

```typ
== Goals

+ #Helps(adopter, to: guide-the-implementer)[because one block carries the whole goal set].
+ #Hinders(maintainer, from: keep-changes-safe)[because a hand-merged marker fails the gate].
+ #Neutral(read-tasks-at-a-glance, run-checks-at-the-write, keep-the-engine-agnostic, adopt-in-one-line, beat-markdown, keep-plain-files, explain-itself, match-docs-to-code).
```

== Changes

- `truth/prelude/2/lib.typ`: generation 1 plus `#import "/truth/goal-calls.typ": *`. Exports are append-only.

```typ
// Generation 2: goal exports are append-only; a rename keeps an alias.
// It imports the live goal-calls.typ, so a new goal changes what the check
// demands of an open task. Append-only covers renames, not additions. A task
// that must not move pins generation 1.
```
- `truth/goal-calls.typ`, `truth/goals.typ`, `truth/process-goals.typ`, `truth/lib.typ`: the chosen names, re-export, aliases to the new keys, keep `check-goal-over` and `surfaced-by`, delete `check-goals`, `_check-group` and `_choices`, and rewrite the module comment. `truth/lib.typ` narrows to `#import "@local/mindtape:0.3.0": edge`. The pin stays: 603 tasks import it.
- `.mindtape/config.toml`, `crates/mindtape/src/rules/mod.rs`: `emit-imports` on `[tasks]`, `give-every-goal-verdict` switched to `check-goal-over` and `markers`, `markers` reserved. Replace the first rule, rewrite its comment, and update the module doc's example.

```toml
[[tasks.rules]]
name = "give-every-goal-verdict"
when = "!terminal && !has(x-goals)"
check = { typst = "truth/lib.typ", function = "check-goal-over", field = "markers" }
severity = "error"

[[tasks.body.headers.rules]]
name = "allow-only-declared-sections"
action = "allowlist"
allowed = [
  { match = "^(AS-IS|Environment|TO-BE|Source of truth|Changes|Alternatives|Rollback|Out of scope|Dogfood|Goals)$" },
]
when = "!terminal"
message = "limit an open task to AS-IS, TO-BE, Changes, Out of scope, Dogfood and Goals, plus its kind's own section"

[[tasks.body.headers.rules]]
name = "keep-one-goals-section"
path = ["Goals"]
action = "unique"
when = "!terminal && !has(x-goals)"
message = "rate every goal under one `== Goals` heading"

[[tasks.body.headers.rules]]
name = "put-goals-after-dogfood"
action = "order"
before = ["Dogfood"]
after = ["Goals"]
when = "!terminal"
message = "move Goals after Dogfood"

# A content rule does not fire on a missing section, so `keep-one-goals-section`
# is what requires one; this rule only refuses an empty one.
[[tasks.body.content.rules]]
name = "write-goals-section"
path = ["Goals"]
assert = "non-empty"
when = "!terminal && !has(x-goals)"
message = "write the goal block under `Goals`; an empty section rates nothing"

[[tasks.body.content.rules]]
name = "cap-goals-at-160-words"
path = ["Goals"]
assert = { words = { max = 160, count = "text" } }
when = "!terminal && !has(x-goals)"
message = "cut the goal reasons to under 160 words"

[[tasks.rules]]
name = "warn-on-x-goals-on-open-tasks"
when = '!terminal && matches(dir, "^truth/tasks/")'
assert = "miss(x-goals)"
message = "move `x-goals` into `== Goals`; the goal check reads markers. A reopened legacy task warns until it is re-rated."
severity = "warn"
```

- `truth/tasks/`: every open task, fourteen today: five outside this milestone, its eight files and the filed follow-up. Rewrite the import to generation 2, delete `x-goals`, append `Goals`. Closed tasks untouched. The milestone's own files are in the set, or `mt check` fails.

```text
truth/tasks/2026-10/decide-whether-jscpds-clone-classes-replace-the-repeats-arm-with-a-gate.typ
  drop #import "@local/mindtape:0.3.0": *
  drop #import "/truth/lib.typ": *
```

This leaf renames every key. A migrated task takes the new name, not an alias:

```text
readable-tasks      -> read-tasks-at-a-glance
implementable-tasks -> guide-the-implementer
enforced-practice   -> run-checks-at-the-write
config-over-code    -> keep-the-engine-agnostic
cheap-adoption      -> adopt-in-one-line
typst-over-markdown -> beat-markdown
plain-files         -> keep-plain-files
self-describing     -> explain-itself
sound-codebase      -> keep-changes-safe
docs-match-code     -> match-docs-to-code
```
- `crates/mindtape-cli/tests/it/check_goals_e2e.rs`, `crates/mindtape-cli/tests/it/main.rs`: add the file and declare it. One breach per rule, a reopened legacy passing, an alias emitting the new key.
- `docs/src/agents.typ`, `AGENTS.md`: generation 2 and the new form.
- `www/content/packs.typ`: `allow-only-declared-sections` reads `project`. Rewrite the `packs-origin` paragraph and `parts` entry, and regenerate `www/generated/packs-origin.json`.
- `www/content/tie-work-to-goals.typ`, `www/content/self-validating-tasks.typ`: a note, not a rewrite. They render `examples/api-service` and `examples/self-validating/`, which keep `x-goals`. Their slices still print it. `tie-work-to-goals.typ` line 24 describes the example's `check-goals`.
- `www/content/filters.typ`, `skills/mindtape/references/filters.md`: `x-goals` stays the hyphenated-name example.
- `schema/output/config-show/1.schema.json`, `www/generated/**`, `.config/outdatty.lock`, `CHANGELOG.md`: regenerate, re-record, add the entry.

```text
contrib/jq/fixtures/config.resolved.json
contrib/jq/fixtures/essentials.out.json
```

== Out of scope

- Closed tasks. A reopened task with `x-goals` may keep it, and warns on every `mt check` until it is re-rated. The warning is the nudge, not a failure. One without the field needs `Goals`.
- The templates, the scaffold and the sweep arms. The next leaf.
- A path-set warning that guesses an audience from `derived.changes`.
- Showing markers in `mt eval --json` or `mt ls --json`.
- Migrating the example projects. They keep the adopter `x-goals` form.
- A goal rename after this commit. A task-facing name is immutable. A closed task calls it. A rename adds the new function and keeps the old one as a one-line alias.
- Extending `leave-no-question-open` to `Goals`. A reason justifies a verdict. It does not specify work.

== Dogfood

`mt ls 'has(x-goals)'` lists only closed tasks and any reopened legacy task the warning names, and `mt ls --json | mt import` rebuilds a task that still evaluates.

== Goals

+ #Helps(maintainer, to: keep-changes-safe)[the migration is what makes the marker check live].
+ #Helps(maintainer, to: match-docs-to-code)[AGENTS and the task files show the same form].
+ #Neutral(read-tasks-at-a-glance, guide-the-implementer, run-checks-at-the-write, keep-the-engine-agnostic, adopt-in-one-line, beat-markdown, keep-plain-files, explain-itself).
