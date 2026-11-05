#import "@local/mindtape:0.4.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Show the distinct-file union in mt eval",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.95,
    ease: 8,
  ),
  difficulty: 1,
  links: (
    (
      task: "add-ast-grep-queries-as-a-shared-query-engine.typ",
      kind: "surfaced-by",
      note: none,
    ),
  ),
  tags: ("found", "cli", "stateless"),
  x-goals: (
    product: verdicts(
      (
        readable-tasks: helps[a reader can reconcile the rows with the printed total],
      ),
      ..(
        "implementable-tasks",
        "enforced-practice",
        "cheap-adoption",
        "typst-over-markdown",
        "plain-files",
      ),
    ),
    process: verdicts(
      (
        docs-match-code: helps[the page and the command explain the same distinct-file rule],
      ),
      ..("sound-codebase",),
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 11, 5),
)

== AS-IS

+ Reviewing both waves of the ast-grep milestone after it landed
+ 🔴 #raw("crates/mindtape-cli/src/cmd/eval.rs"): The `mt eval` breakdown prints each query's file count but not the distinct union, so the rows can sum past the total.

A task estimated `count(1, grep(`warm`), grep(`fn`, "src/**"))` where both queries match the same two files:

```
$ mt eval two
estimate.files = 3
  1        manual
  2 files  grep `warm`
  2 files  grep `fn` in `src/**`
```

The rows sum to 5, the total is 3, and nothing says a file matched twice. `crates/mindtape-cli/src/cmd/eval.rs` prints each match count and never the union.

== TO-BE

- 🟢 When the per-query counts exceed the union, `mt eval` prints the distinct union.
- 🟢 A task whose queries do not overlap keeps its current output.

```
$ mt eval two
estimate.files = 3
  1        manual
  2 files  grep `warm`
  2 files  grep `fn` in `src/**`
  2 files  distinct across all queries
```

== Changes

- `crates/mindtape-cli/src/cmd/eval.rs`: add the distinct note when `matches.iter().sum() > total - manual`.
- `crates/mindtape-cli/src/cmd/eval_tests.rs`: a red test for overlapping queries and a no-note test for disjoint ones.
- `www/content/estimates.typ`: say the note appears when queries overlap.

== Out of scope

- Changing the `estimate_counts` JSON shape.
- A per-file listing in `mt eval`.

== Dogfood

This repo's own multi-query units are reconcilable from `mt eval` without reading `mt eval --json`.
