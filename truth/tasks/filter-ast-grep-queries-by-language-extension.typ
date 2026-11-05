#import "@local/mindtape:0.4.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Filter ast-grep queries by the language's file extensions",
  kind: "dev",
  priority: ice(
    impact: 9,
    confidence: 0.98,
    ease: 7,
  ),
  difficulty: 3,
  links: (
    (
      task: "add-ast-grep-queries-as-a-shared-query-engine.typ",
      kind: "surfaced-by",
      note: none,
    ),
  ),
  tags: ("found", "mt", "stateless"),
  x-goals: (
    product: verdicts(
      (
        enforced-practice: helps[a shape count reads only the language's source files, so a rule can trust it],
      ),
      ..(
        "readable-tasks",
        "implementable-tasks",
        "cheap-adoption",
        "typst-over-markdown",
        "plain-files",
      ),
    ),
    process: verdicts(
      (
        sound-codebase: helps[the shared engine filters by the grammar's own extension map instead of a second walk],
      ),
      ..("docs-match-code",),
    ),
  ),
  assignee: "agent",
  status: done(2026, 11, 5),
)

== AS-IS

+ Reviewing both waves of the ast-grep milestone after it landed
+ 🔴 #raw("crates/mindtape/src/query/mod.rs"): `ast_grep` parses every git-listed UTF-8 file as Rust, so a broad query counts Markdown, TOML and the task file that declares it.

A temp git project held `README.txt`, `docs/notes.md`, `src.rs` and the task's own `tasks/a.typ`, each containing `warm(1)`. The task estimated:

```
estimate: (files: count(1, ast_grep("rust", `warm($$$ARGS)`))),
```

```
$ mt eval a
estimate.files = 5
  1        manual
  4 files  ast-grep rust `warm($$$ARGS)`

$ mt eval a --json | jq .derived
{"callers": ["README.txt", "docs/notes.md", "src.rs", "tasks/a.typ"]}
```

`crates/mindtape/src/query/mod.rs` reads every candidate and runs the Rust grammar over it; no extension check exists.

== TO-BE

- 🟢 Only files with the language's extensions are parsed, `.rs` for `rust` and `rs`.
- 🟢 A `.md` or `.typ` file with the same text matches nothing, and a task file that declares the query no longer matches itself.
- 🟢 The estimates page and the collector pages say which files a language query reads.
- 🟢 A red test pins a `.rs` match beside `.md` and `.typ` decoys.

```
$ mt eval a
estimate.files = 2
  1        manual
  1 file   ast-grep rust `warm($$$ARGS)`
```

== Changes

- `crates/mindtape/src/query/mod.rs`: carry the language beside the compiled pattern and filter candidates by `SupportLang::from_path`.
- `crates/mindtape/src/query/tests.rs`: a `.rs` file matches while a `.md` and `.typ` with the same text do not.
- `crates/mindtape/src/collect/tests.rs` and `crates/mindtape/src/estimate_queries_tests.rs`: one red test each that a non-Rust file is not counted.
- `www/content/estimates.typ`, `config.typ`, `rules.typ`: name the extension rule.
- `CHANGELOG.md`: say a shape query reads only its language's files.

== Out of scope

- Extra grammars; this wave stays Rust-only.
- Matching macro expansions; ast-grep reads source, not expanded code.
- A persistent parse cache.

== Dogfood

The estimates fixture's shape query counts only its `src/*.rs` files, and the task that declares the query no longer matches itself.
