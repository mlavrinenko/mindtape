#import "@local/mindtape:0.3.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Add derived task fields and a regex collector",
  kind: "feat",
  priority: ice(
    impact: 8,
    confidence: 0.9,
    ease: 5,
  ),
  difficulty: 2,
  tags: ("mt", "gating", "stateless"),
  estimate: (src-files: 22),
  x-goals: (
    product: verdicts(
      (
        implementable-tasks: helps[the task's own Changes plan feeds the size rule, with no second estimate to keep in sync],
        enforced-practice: helps[a pack ships a collector and its return type together, so a rule reads a checked contract],
      ),
      ..(
        "readable-tasks",
        "cheap-adoption",
        "typst-over-markdown",
        "plain-files",
      ),
    ),
    process: verdicts(
      (
        sound-codebase: helps[the collector, its type check and both aggregates land with red tests],
        docs-match-code: helps[the config schema and mt rules show each collector's shape and origin],
      ),
      ..(),
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 11, 4),
)
== AS-IS

+ Run `mt ls 'has(estimate.src-files)'` in this repo.
+ 🔴 Every open task carries an estimate that restates the files in `== Changes`, and the size rule reads the estimate while the body plan is unchecked.
+ Edit a task's Changes to name seven source files and leave `estimate: (src-files: 1)`; `mt check` passes.
+ Open `crates/mindtape/src/packs/budget.toml`.
+ 🔴 `judgment-stays-small` reads `estimate.src-files`, so the plan a reader sees and the number a rule enforces are two separate declarations.


== TO-BE

- 🟢 `[[tasks.collectors]]` entries define a named collector; the name is the derived field and `when` gates it.
- 🟢 `query` is a regex over a body section. The result is read as `derived.<name>` by filters, `--sort`, `mt eval` and `mt ls --json`.
- 🟢 `returns` is a type expression over `bool`, `int`, `number`, `string`, `path` and `date`, an array `[T]`, or a record `{ field = T }`.
- 🟢 `mt rules` lists each collector with its origin, kind, type and rule readers; packs merge collectors by name.
- 🟢 `len(field)` counts list items; `count_paths(field, ...sets)` counts distinct normalized paths in one or more path sets.

```toml
[[tasks.collectors]]
name = "changes"
query = { section = "Changes", pattern = "..." }
returns = ["path"]
```


== Changes

- `crates/mindtape/src/config/collect.rs` (new): `[[tasks.collectors]]`, `CollectorSpec`, `section` (a heading path or body segment), `pattern`/`group`, the `returns` type expression and load-time validation. A regex returns `["string"]` or `["path"]`. A filter naming an undeclared `derived.<name>` fails load.
- `crates/mindtape/src/collect/mod.rs` (new): run collectors after load and on a candidate. A regex takes each match's first capture group, or the whole match, from every matching section, sliced as the body content gate does. A missing section is an empty list; a gated task has no field.
- `crates/mindtape/src/model/{mod,canonical,fields}.rs`: `Task.derived`, a read-only `derived` JSON map, dynamic `derived.<name>` and record reads, `len` on any list and `count_paths` on `["path"]`; a missing field is Missing.
- `crates/mindtape/src/filter/{mod,predicates,eval,compare}.rs` and `crates/mindtape/src/sort.rs` with `config/{semantic,views,ls}.rs`: read derived values; a scalar compares and sorts, a list or record fails both at load.
- `crates/mindtape/src/config/{tasks,packs,origins,pack_registry,mod}.rs` and `lib.rs`: wire, export, merge by name, `disable`, fail duplicates, record origins.
- `crates/mindtape/src/loader/mod.rs`, `crates/mindtape/src/validate/candidate.rs`: annotate loaded and candidate tasks; `mt import` runs collectors but ignores their values.
- `crates/mindtape-cli/src/cmd/{eval,rules}.rs`: `eval` loads the project set when found, fills the body and collects; without a project it uses an empty set and refuses a `when` that needs one. `rules` prints the derived block even with no rules; readers are direct `derived.` names in `[[tasks.rules]]`, and `--for commit` hides it.
- `crates/mindtape/src/config/commits.rs`: refuse `derived.` in `[[commits.rules]]`.
- `crates/mindtape/src/collect/tests.rs`, `config/collect_tests.rs`, `filter/derived_tests.rs`: red tests first.
- `schema/config.schema.json`, `www/content/{config,filters,rules,schema}.typ`, `CHANGELOG.md`: schema and docs.


== Alternatives

- A separate `[[tasks.derive]]` binding: one concept too many; the collector name is the field name.
- `returns = "paths"` as a named shape: it fits only a list of paths; a type expression covers scalars, lists and records.
- A new `mt derive` command: one more command to discover; fold the listing into `mt rules`.
- A regex-only design: a project still needs a Typst escape hatch for structured extraction.


== Out of scope

- A Typst function collector: `add-a-typst-function-collector.typ`, which depends on this task.
- The `estimate.src-files` migration: `replace-estimate-src-files-with-a-collected-changes-count.typ`.
- ast-grep file queries: the milestone `add-ast-grep-queries-as-a-shared-query-engine.typ`.
- Regex over a field other than a section; the Typst kind covers those.
- Commit-scope rules reading derived fields; v1 refuses the name at load.
- A unified `mt info` command over rules, tags and views.


== Dogfood

After the migration, `judgment-stays-small` reads `derived.changes` from this repo's Changes sections, and `mt rules` shows the collector and its readers.
