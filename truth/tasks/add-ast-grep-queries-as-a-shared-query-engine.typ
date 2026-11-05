#import "@local/mindtape:0.3.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Add ast-grep queries as a shared query engine",
  priority: ice(
    impact: 6,
    confidence: 0.7,
    ease: 3,
  ),
  difficulty: 5,
  kind: "milestone",
  tags: ("mt", "tooling", "gating", "stateless"),
  estimate: (src-files: 0),
  x-goals: (
    product: verdicts(
      (
        implementable-tasks: helps[an estimate counts real call sites instead of a text proxy],
      ),
      ..(
        "readable-tasks",
        "enforced-practice",
        "cheap-adoption",
        "typst-over-markdown",
        "plain-files",
      ),
    ),
    process: verdicts(
      (
        sound-codebase: helps[the query reuses the estimate query cache and path sets instead of adding a second engine],
      ),
      ..("docs-match-code",),
    ),
  ),
  assignee: "agent",
  status: done(2026, 11, 5),
  links: () + depends-on("add-derived-task-fields-and-a-regex-collector.typ"),
)

== AS-IS

+ Open `crates/mindtape/src/estimate_queries.rs`.
+ 🔴 A unit takes a manual number or `count(n, grep(...))`; the queries see text only.
+ Add a task whose estimate should count the files that call `parse_task`.
+ 🔴 A rename or a macro hides a site from `grep`, and no query can ask the code's shape.


== TO-BE

- 🟢 A shared file-query engine in `crates/mindtape/src/query/mod.rs` owns the query value, compile, git-listed file walk, path filter and process cache; `estimate_queries` becomes its first consumer.
- 🟢 `ast_grep("rust", "parse_task($$$ARGS)", "crates/**")` compiles into that engine, runs through `ast-grep-core` and the Rust grammar, and counts distinct files in the unit's path set.
- 🟢 Other languages arrive as opt-in grammar crates; a Typst grammar is a later leaf for task-file queries.
- 🟢 A bad pattern or unknown language leaves the unit unresolved with a `problem`, as a bad `grep` does.
- 🟢 A later collector leaf exposes the same engine through `[[tasks.collectors]]`, so a rule reads a code fact as `derived.<name>`.

```typ
estimate: (src-files: count(0, ast_grep("rust", "parse_task($$$ARGS)", "crates/**")))
```


== Changes

- `crates/mindtape/Cargo.toml`: leaf 1, spike and seam, pins `ast-grep-core` with `ast-grep-language` and `tree-sitter-rust`; `crates/mindtape/src/query/mod.rs` (new) extracts `Query`, compile, file walk, path filter and cache from `estimate_queries`; a size/cross-build note in `docs/src/contributing/release.typ`.
- `lib/prelude.typ`: leaf 2, helper and prelude, plus `lib/typst.toml`, `crates/mindtape/src/loader/preludes/0.4.0/prelude.typ`, `embedded.rs`, `embedded_tests.rs`, `emit/mod.rs`, `test_support/fs.rs`, `crates/mindtape-cli/tests/it/support/fixtures.rs`: `ast_grep(...)`, `_fmt-query`, and the 0.4.0 version bump.
- `crates/mindtape/src/query/mod.rs`: leaf 3, estimate query, plus `estimate_queries/mod.rs`: the `Query::AstGrep` variant with canonical tag `ast-grep`, label, compile, match, optional glob and distinct files.
- `crates/mindtape/src/model/canonical.rs`: leaf 4, forms, plus `emit/mod.rs`, `import_map/mod.rs`: keep the source form through `mt ls --json` and `mt import`, and update the version lists in `docs/src/agents.typ`, `www/content/config.typ`, `task-files.typ`.
- `crates/mindtape/src/config/collect.rs`: leaf 5, collector consumer, plus `collect/astgrep.rs`: a `query = { ast_grep = ... }` variant that reuses the engine; decide the task-side input and whether a Typst grammar is needed. Depends on the derived-fields task.
- `crates/mindtape-cli/tests/it/estimate_queries_e2e.rs`: leaf 6, tests and docs, plus `www/content/estimates.typ`, `CHANGELOG.md`, `www/generated/*`.


== Out of scope

- Running an external `sg` command by default: a sandboxed in-process query is the default; a shell-out needs `mt allow`.
- A persistent parse cache: the process cache is enough until a profile says otherwise.


== Dogfood

This repo's judgment tasks can estimate a rename or a trait change by matching real call sites, and a collector can expose the same count as `derived.<name>`.
