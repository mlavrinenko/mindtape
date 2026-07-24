#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Conditional ls rules",
  priority: ice(
    impact: 6,
    confidence: 0.7,
    ease: 4,
  ),
  tags: ("cli",),
  links: depends-on("default-view-for-ls.typ"),
  status: done(
    2026,
    7,
    12,
  )[shipped [[ls.rules]] + condition language + JSON envelope],
)

== Summary

Make bare `mt ls` adapt to ambient project state through ordered `[[ls.rules]]`.
First match wins — exactly one rule applies; it composes its facets from views
(`import-views`) plus manual overrides. No cross-rule cascade, no `stop` flag.
Rule conditions use a filter-syntax mini-language over environment predicates
(branch, env, date/time), so the feature works with or without git. This is the
ambient auto-activation reframed off the retired "context" concept — `[ls]`
policy, not a stateful mode. MindTape stays project-scoped.

Auto-activation applies only when the caller specifies no filter and no `@view`.

== Config

```toml
[[ls.rules]]
name = "release"                              # optional; shown in the trailer / JSON
condition = 'on_branch("release/*")'          # filter-syntax over env predicates; absent = always
import-views = ["inbox", "release(sort)"]     # compose left->right, per-facet last-wins
filter = '!terminal && due <= "2026-08-01"'   # optional manual override (highest precedence)
```

Rule fields, all optional:

- `name` — shown in the trailer and JSON; falls back to the condition text when
  unset.
- `condition` — filter-syntax expression over environment predicates; absent
  matches always.
- `import-views` — ordered list. Each entry is `"view"` (all facets) or
  `"view(facet, ...)"` (a subset of `filter` / `sort` / `columns`). Composed
  left-to-right, per-facet last-wins. Bare name = all three; empty parens is an
  error.
- `filter` / `sort` / `columns` — manual facet overrides (same grammar as
  `[views]`); highest precedence. `filter` may use `@view` / `in_view`;
  `sort` / `columns` may splice `@view`.

== Resolution

Bare `mt ls` (no CLI filter, no `@view`):

+ Explicit CLI filter or `@view` -> used verbatim; rules and `default-view`
  skipped.
+ Else the first matching `[[ls.rules]]` -> `compose(import-views)` then manual
  overrides.
+ Else `default-view` (from `default-view-for-ls`).

Per facet inside a matched rule, precedence high-to-low: a manual field > the
last `import-views` entry that provides it > the built-in default. `default-view`
is NOT a base layer beneath a matched rule — a rule composes everything it uses
(re-import the default view explicitly if wanted).

== Condition language (`crates/mindtape/src/condition.rs`)

Same evalexpr parser and operators as filters (`== != < > <= >= && || !`,
parens, bare-or-call predicates), a separate evaluation domain: the environment,
no task. A task field in a condition is a load-time error.

Predicates / values — initial, complete set:

- `on_branch("glob")` -> bool. Glob-match the current git branch (reuse the
  derive-tags glob matcher). False when not a git repo / detached HEAD.
- `env("NAME")` -> string (`""` when unset).
- `weekday()` -> int 1..7 (Mon..Sun).
- `now()` -> ISO date string (reused from `filter.rs`).

Comparisons take a value or literal on either side (`env("MODE") == "review"`,
`now() >= "2026-07-01"`, `weekday() >= 6`). Glob only for `on_branch`; regex is a
future `on_branch_re` predicate, out of scope.

Purity: `condition.rs` is pure — environment facts (`branch: Option<String>`,
env lookups, today) injected as a struct (mirror `worktrees.rs`'s injected-fs
pattern), consumer-blind and unit-testable; the CLI populates the struct.

Shared helpers: factor `unwrap_root`, `extract_literal`, `extract_identifier`,
the comparison-operator mapping, and `now()` handling out of `filter.rs` into a
shared module (e.g. `expr.rs`) consumed by both `filter.rs` and `condition.rs` —
do NOT copy them (the jscpd >=70-token gate rejects a duplicated block).

== Git HEAD reader

A small reader resolving the current branch from the project root's `.git`: parse
`.git/HEAD` (`ref: refs/heads/<name>`), a detached HEAD -> `None`, and a worktree
`.git` file (`gitdir: <path>`) -> read `<path>/HEAD`. Used by `on_branch`. No new
dependency.

== Trailer

After the table (human formats only, suppressed under `--json` / `--csv`), a dim
line attributing each facet to its winning source, grouped in application order:

```
applied: inbox (filter, columns) -> release (sort)
```

Source = a view name, or the rule name / `manual` when a manual field wins.
Facets left at built-in defaults are omitted. Shown only when a rule fired or a
non-trivial `default-view` is active.

== JSON envelope (breaking)

Convert `mt ls --json` from a bare task array to an object:

```json
{
  "tasks": [ ... ],
  "applied": {
    "rule": "release",
    "facets": {
      "filter":  { "from": "inbox",   "value": "..." },
      "sort":    { "from": "release", "value": ["priority:desc"] },
      "columns": { "from": "inbox",   "value": ["#", "status", "due", "title"] }
    }
  }
}
```

Fallback (no rule matched): `"rule": null`, and each facet `from` = the
`default-view` name or `"match-all"`. Update every consumer: `skills/mindtape/`
`SKILL.md`, www docs, CHANGELOG; grep for jq consumers.

== Validation (config load)

- Each rule `condition` parses in the condition grammar and uses only environment
  predicates.
- `import-views` entries resolve to declared views; facet selectors are a subset
  of `filter` / `sort` / `columns`; empty parens error.
- Manual `filter` / `sort` / `columns` are valid (reuse existing validators);
  `@view` references resolve; the combined view-reference graph stays acyclic
  (extend the `config_views` cycle check).

== Tests

Condition eval per predicate (`on_branch` glob including no-git, `env`,
`weekday`, `now` compare); first-match order; `import-views` composition + facet
selectors + per-facet precedence under a manual override; `default-view` fallback
when no rule matches; explicit filter / `@view` bypass; trailer attribution; JSON
envelope shape; load-time validation errors (unknown view, bad facet,
task-field-in-condition, cycle); `.git/HEAD` parse including detached + worktree
file.

== Gates

New modules keep `filter.rs` and `config_ls.rs` under the 500-line limit; eject
inline tests (`ejectest`) as needed. Run `just check` on the integrated tree.
