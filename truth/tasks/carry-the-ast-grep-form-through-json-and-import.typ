#import "@local/mindtape:0.3.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Carry the ast-grep form through JSON and import",
  kind: "feat",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 3,
  tags: ("mt", "tooling", "stateless", "no-repro"),
  x-goals: (
    product: verdicts(
      (
        plain-files: helps[the query survives a JSON round trip instead of living only in a Typst file],
      ),
      ..(
        "readable-tasks",
        "implementable-tasks",
        "enforced-practice",
        "cheap-adoption",
        "typst-over-markdown",
      ),
    ),
    process: verdicts(
      (
        sound-codebase: helps[one canonical enum carries the authored query in both directions],
      ),
      ..("docs-match-code",),
    ),
  ),
  assignee: "agent",
  status: done(2026, 11, 5),
  links: parent("add-ast-grep-queries-as-a-shared-query-engine.typ")
    + depends-on("add-the-ast-grep-estimate-query.typ"),
)

== AS-IS

+ Export a task whose estimate uses `ast_grep(...)` with `mt ls --json`.
+ 🔴 `CanonicalQuery` has `Grep` and `Glob` only, so the ast-grep query cannot serialize.
+ Re-import the export and the shape query is lost.


== TO-BE

- 🟢 `CanonicalQuery::AstGrep` serializes with the tag `ast-grep` and carries `language`, `pattern`, optional `glob` and read-only `files`.
- 🟢 `mt import` rebuilds `Query::AstGrep`, so a JSON round trip keeps the language, pattern and glob.
- 🟢 `emit` writes `ast_grep("rust", `pattern`, "glob")` back as an authored call.
- 🟢 The prelude version lists in the agent map, config page and task-files page name 0.4.0.

```json
{
  "query": "ast-grep",
  "language": "rust",
  "pattern": "parse_task($$$ARGS)",
  "glob": "crates/**",
  "files": 3
}
```


== Changes

- `crates/mindtape/src/model/canonical.rs`: the `AstGrep` variant, its serde tag and both conversions, which `import_map` reads to rebuild the query.
- `crates/mindtape/src/emit/mod.rs`: write `ast_grep(...)` back.
- `crates/mindtape/src/emit/tests.rs`: red round-trip test for the authored form.
- `crates/mindtape-cli/tests/it/estimate_queries_e2e.rs`: `ls --json | import` keeps the shape query.
- `docs/src/agents.typ`: the prelude version list.
- `www/content/config.typ`: the prelude version list.
- `www/content/task-files.typ`: the prelude version list.


== Alternatives

- Reuse `CanonicalQuery::Grep` with a language field: changes what an existing tag means and leaves an old consumer guessing.
- Carry the source as a free-form string: import would have to parse Typst again.


== Out of scope

- The `[[tasks.collectors]]` consumer.
- Extra language grammars and a Typst grammar.


== Dogfood

An agent can export, batch-edit and re-import a shape query without losing its language, pattern or glob.

