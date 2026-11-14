#import "@local/mindtape:0.3.0": *

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
  )[landed `[[ls.rules]]`, the condition language and the JSON envelope],
)

== Summary

Make bare `mt ls` adapt to ambient project state through ordered `[[ls.rules]]`.
First match wins: exactly one rule applies. It composes its facets from views
(`import-views`) plus manual overrides. No cross-rule cascade, no `stop` flag.
Rule conditions use a filter-syntax mini-language over environment predicates
(branch, environment, date/time). The feature then works with or without git.
This is the ambient auto-activation moved off the retired "context" concept:
`[ls]` policy, not a stateful mode. The tool stays project-scoped.

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

- `name`: shown in the trailer and JSON, and falls back to the condition text
  when unset.
- `condition`: filter-syntax expression over environment predicates. Absent
  matches always.
- `import-views`: ordered list. Each entry is `"view"` (all facets) or
  `"view(facet, ...)"` (a subset of `filter` / `sort` / `columns`). Composed
  left-to-right, per-facet last-wins. A bare name means all three, and empty
  `parens` is an error.
- `filter` / `sort` / `columns`: manual facet overrides (same grammar as
  `[views]`), with the highest precedence. `filter` may use `@view` / `in_view`.
  `sort` / `columns` may splice `@view`.

== Resolution

Bare `mt ls` (no CLI filter, no `@view`):

+ Explicit CLI filter or `@view` -> used verbatim. Rules and `default-view`
  do not apply.
+ Else the first matching `[[ls.rules]]` -> `compose(import-views)` then manual
  overrides.
+ Else `default-view` (from `default-view-for-ls`).

Per facet inside a matched rule, precedence high-to-low: a manual field > the
last `import-views` entry that provides it > the built-in default. `default-view`
does not layer beneath a matched rule. A rule composes everything it uses, so
re-import the default view explicitly if wanted.

== Condition language (`crates/mindtape/src/condition.rs`)

Same `evalexpr` parser and operators as filters (`== != < > <= >= && || !`,
`parens`, bare or call predicates), a separate evaluation domain: the environment,
no task. A task field in a condition is a load-time error.

Predicates / values, initial and complete:

- `on_branch("glob")` -> `bool`. Glob-match the current git branch (reuse the
  derive-tags glob matcher). False when not a git repo / detached HEAD.
- `env("NAME")` -> string (`""` when unset).
- `weekday()` -> int 1..7 (Mon..Sun).
- `now()` -> ISO date string (reused from `filter.rs`).

Comparisons take a value or literal on either side (`env("MODE") == "review"`,
`now() >= "2026-07-01"`, `weekday() >= 6`). Glob applies only to `on_branch`. Regex is a
future `on_branch_re` predicate, out of scope.

Purity: `condition.rs` is pure. Environment facts (`branch: Option<String>`,
`env` lookups, today) go in as a struct, mirroring `worktrees.rs`'s
injected-fs pattern. It stays consumer-blind and unit-testable, and the CLI
populates the struct.

Shared helpers: factor `unwrap_root`, `extract_literal`, `extract_identifier`,
the comparison-operator mapping and `now()` handling out of `filter.rs` into a
shared module such as `expr.rs`. Both `filter.rs` and `condition.rs` consume it.
Do not copy them, because the jscpd >=70-token gate rejects a duplicated block.

== Git branch lookup

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
Facets left at built-in defaults do not appear. Shown only when a rule fired or a
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
`SKILL.md`, the `www` docs, and CHANGELOG. Grep for jq consumers.

== Validation (config load)

- Each rule `condition` parses in the condition grammar and uses only environment
  predicates.
- `import-views` entries resolve to declared views, with facet selectors a
  subset of `filter` / `sort` / `columns`. Empty `parens` is an error.
- Manual `filter` / `sort` / `columns` are valid (reuse existing validators).
  `@view` references resolve, and the combined view-reference graph stays acyclic
  (extend the `config_views` cycle check).

== Tests

Condition eval per predicate (`on_branch` glob including no-git, `env`,
`weekday`, `now` compare). First-match order. `import-views` composition, facet
selectors, and per-facet precedence under a manual override. `default-view`
fallback when no rule matches. Explicit filter or `@view` bypass. Trailer
attribution and JSON envelope shape. Load-time validation errors (unknown view,
bad facet, task-field-in-condition, cycle). `.git/HEAD` parse including detached
and worktree file.

== Gates

New modules keep `filter.rs` and `config_ls.rs` under the 500-line limit. Eject
inline tests (`ejectest`) as needed. Run `just check` on the integrated tree.
