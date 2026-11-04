#import "@local/mindtape:0.3.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Add the ast-grep estimate query",
  kind: "feat",
  priority: ice(
    impact: 7,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 4,
  tags: ("mt", "tooling", "stateless", "no-repro"),
  x-goals: (
    product: verdicts(
      (
        implementable-tasks: helps[an estimate counts a real call site instead of a text proxy],
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
        sound-codebase: helps[the shape query compiles and matches through the one engine, never a second file walk],
      ),
      ..("docs-match-code",),
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 11, 5),
  links: parent("add-ast-grep-queries-as-a-shared-query-engine.typ")
    + depends-on("add-the-ast-grep-prelude-helper-and-prelude-0-4-0.typ"),
)

== AS-IS

+ Open `crates/mindtape/src/query/mod.rs` after the engine extraction lands.
+ 🔴 It has `Grep` and `Glob` only, so `ast_grep(...)` parses as no query and `mt check` refuses the estimate.
+ A text regex cannot see a call site hidden by a macro or a rename.


== TO-BE

- 🟢 `Query::AstGrep { language, pattern, glob }` parses the `ast-grep` dict, labels itself, compiles the grammar and pattern, and matches files.
- 🟢 Only `rust` (alias `rs`) is a language in this wave; any other name leaves the unit unresolved with a problem naming the supported set.
- 🟢 A bad pattern leaves the unit unresolved with a problem, never a panic.
- 🟢 A file matching twice counts once, and the count is the distinct files in the unit's path set.

```typ
estimate: (src-files: count(0, ast_grep("rust", "parse_task($$$ARGS)", "crates/**")))
```

```text
estimate.src-files = 3
  0        manual
  3 files  ast-grep rust `parse_task($$$ARGS)` in `crates/**`
```


== Changes

- `crates/mindtape/src/query/mod.rs`: the `AstGrep` variant, parse, label, compile, match and distinct-file count.
- `crates/mindtape/src/query/tests.rs`: red tests for parse, a bad pattern, an unknown language, a double match and the glob scope.
- `crates/mindtape/src/estimate_queries_tests.rs`: the shared engine still resolves greps and globs.
- `crates/mindtape-cli/tests/it/estimate_queries_e2e.rs`: the e2e proof through the real prelude.


== Alternatives

- Shelling out to `sg`: needs `mt allow`, gives a different result on each host and re-parses the tree in another process.
- Falling back to a regex when the grammar cannot run: a silent text proxy is the failure this task exists to remove.


== Out of scope

- The canonical JSON and import form, which land in the next leaf.
- The `[[tasks.collectors]]` consumer.
- Extra language grammars and a Typst grammar.


== Dogfood

This repo can size a `parse_task` rename by its real call sites instead of a grep.

