#import "@local/mindtape:0.3.0": *
#import "/truth/lib.typ": *

#show: task.with(
  title: "Extract the shared query engine and pin ast-grep",
  priority: ice(
    impact: 7,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 4,
  kind: "refactor",
  tags: ("mt", "tooling", "stateless", "no-repro"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: helps[a later collector reuses one engine instead of a second file walk],
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: helps[the query value, compile, walk, path filter and cache live in one module],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 11, 5),
  links: parent("add-ast-grep-queries-as-a-shared-query-engine.typ"),
)

== AS-IS

+ Open `crates/mindtape/src/estimate_queries.rs`.
+ 🔴 The query value, git-listed file walk, `PathFilter` and process cache live there. A second collector then needs a copy of them.
+ There is no ast-grep dependency, so a shape query cannot compile.


== TO-BE

- 🟢 `crates/mindtape/src/query/mod.rs` owns `Query`, compile, the git-listed file walk, `PathFilter` and the process cache.
- 🟢 `estimate_queries` resolves its units through the module, with its tests and e2e unchanged in behavior.
- 🟢 The build pins `ast-grep-core` and `ast-grep-language` with only the Rust grammar. A later leaf then adds `ast_grep(...)` without a second engine.
- 🟢 `docs/src/contributing/release.typ` records the C-backed build and binary-size cost.

```toml
ast-grep-core = "0.50"
ast-grep-language = { version = "0.50", default-features = false, features = ["tree-sitter-rust"] }
```


== Changes

- `crates/mindtape/src/query/mod.rs`: the extracted query value, compile, file walk, path filter and cache.
- `crates/mindtape/src/estimate_queries.rs`: keep `CountEstimate` and `annotate`, and call the engine.
- `crates/mindtape/src/estimate_queries_tests.rs`: move query tests beside the engine and keep the red behavior.
- `crates/mindtape/Cargo.toml`: pin the two crates with the Rust grammar feature.
- `Cargo.lock`: resolve the new crate graph.
- `docs/src/contributing/release.typ`: the size and cross-build note.


== Out of scope

- The `ast_grep(...)` helper and the `Query::AstGrep` variant, which land in the next leaves.
- The `[[tasks.collectors]]` consumer.


== Dogfood

This repo's estimate units keep resolving, and the later leaves reuse this engine. The Windows cross check passes with the C grammar built.

