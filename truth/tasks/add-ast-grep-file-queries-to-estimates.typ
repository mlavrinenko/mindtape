#import "@local/mindtape:0.3.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Add ast-grep file queries to estimates",
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
  status: proposed(2026, 11, 4),
  links: (),
)

== AS-IS

+ Open `crates/mindtape/src/estimate_queries.rs`.
+ 🔴 A unit takes `count(n, grep(...))`, `glob(...)` or a manual number; every query sees text only.
+ Add a task whose estimate should count the files that call `parse_task`.
+ 🔴 A rename or a macro hides a site from `grep`, and no query can ask the code's shape.


== TO-BE

- 🟢 `count(n, ast_grep(lang: "rust", pattern: "parse_task($$$ARGS)", in: "crates/**"))` counts the distinct files the pattern matches inside the unit's path set.
- 🟢 The query runs in process through `ast-grep-core` and the Rust grammar, with other languages as opt-in grammar crates. No shell and no `mt allow`.
- 🟢 The first leaf is a spike that confirms the crate, the grammar features, the binary-size and cross-build cost, and the initial language list.
- 🟢 A bad pattern or an unknown language leaves the unit unresolved with a `problem`, as a bad `grep` does; `mt check` names it.
- 🟢 The query reuses the process cache keyed by root, path set and query; there is no persistent query cache.

```typ
estimate: (src-files: count(0, ast_grep(lang: "rust", pattern: "parse_task($$$ARGS)", in: "crates/**")))
```


== Changes

- Leaf 1, spike: `crates/mindtape/Cargo.toml`, a split `crates/mindtape/src/estimate_queries/mod.rs`, a Rust fixture, and a size/cross-build note in `docs/src/contributing/release.typ`.
- Leaf 2, query helper: `lib/prelude.typ`, `crates/mindtape/src/loader/preludes/0.4.0/prelude.typ`, `crates/mindtape/src/loader/embedded.rs`, `lib/typst.toml`: `ast_grep(...)` beside `grep` and `glob`.
- Leaf 3, engine: `crates/mindtape/src/estimate_queries/mod.rs`: the `Query::AstGrep` variant with canonical tag `ast-grep`, label, compile, match, `in` glob and distinct files, plus the `grep`/`glob` label and error.
- Leaf 4, forms: `crates/mindtape/src/model/canonical.rs`, `emit/mod.rs`, `crates/mindtape-cli/src/cmd/import`: keep the source form through `mt ls --json` and `mt import`.
- Leaf 5, tests and docs: `crates/mindtape-cli/tests/it/estimate_queries_e2e.rs`, `www/content/estimates.typ`, `CHANGELOG.md`, `www/generated/*`.


== Out of scope

- Task-body collectors: regex and Typst collectors read a task, while this query reads code for an estimate.
- Adding a tree-sitter Typst grammar: the task DSL is not queried by ast-grep.
- Running an external `sg` command by default: a sandboxed in-process query is the default; a shell-out needs `mt allow`.
- A persistent parse cache: the process cache is enough until a profile says otherwise.


== Dogfood

This repo's judgment tasks can estimate a rename or a trait change by matching the code's real call sites instead of grepping a symbol name.
