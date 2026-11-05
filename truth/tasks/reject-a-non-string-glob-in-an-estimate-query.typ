#import "@local/mindtape:0.4.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Reject a non-string glob in an estimate query",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.95,
    ease: 8,
  ),
  difficulty: 2,
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
        enforced-practice: helps[a wrong-typed query argument is refused instead of producing a plausible wrong count],
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
      (:),
      ..("sound-codebase", "docs-match-code"),
    ),
  ),
  assignee: "agent",
  status: done(2026, 11, 5),
)

== AS-IS

+ Reviewing both waves of the ast-grep milestone after it landed
+ 🔴 #raw("crates/mindtape/src/estimate_queries.rs"): A non-string third argument to `ast_grep` or `grep` is silently dropped, so the query widens to all files.

A task estimated this in a fixture with two matching `.rs` files:

```
count(1, ast_grep("rust", `warm($$$ARGS)`, 42))
```

```
$ mt eval numglob
estimate.files = 3
  1        manual
  2 files  ast-grep rust `warm($$$ARGS)`
```

No `in ...` scope and no error: `crates/mindtape/src/query/mod.rs` keeps `glob` only when it is a string, and `parse_count` never checks the type.

== TO-BE

- 🟢 A `glob` that is not a string is a shape error at `estimate.<unit>.queries[i]`, naming `glob`.
- 🟢 A list, number or boolean is refused instead of widening the count.
- 🟢 A red test covers `ast_grep` and `grep`.

```
$ mt eval numglob
error: estimate.files.queries[0]: glob must be a string
```

== Changes

- `crates/mindtape/src/estimate_queries.rs`: in `parse_count`, reject a `glob` key whose value is not a string.
- `crates/mindtape/src/estimate_queries_tests.rs`: red tests for a number, a list and a boolean glob.
- `www/content/estimates.typ`: say the third argument is one glob string.

== Out of scope

- Changing the prelude ABI or bumping the prelude version.
- Accepting a list of globs.

== Dogfood

A mistyped glob in this repo's next estimate fails `mt check` instead of counting every file.
