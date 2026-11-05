#import "@local/mindtape:0.4.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Show an integer distinct union note",
  kind: "dev",
  priority: ice(
    impact: 3,
    confidence: 0.98,
    ease: 9,
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
        readable-tasks: helps[the note shows a file count, not an arithmetic artifact],
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
        docs-match-code: helps[the code matches the integer file count the page promises],
      ),
      ..("sound-codebase",),
    ),
  ),
  assignee: "agent",
  status: done(2026, 11, 6),
)

== AS-IS

+ Verifying the fixes for the ast-grep review
+ 🔴 #raw("crates/mindtape-cli/src/cmd/eval.rs"): A fractional manual count makes the distinct union note print a floating-point file count.

A task with this estimate:

```
count(0.3, grep(`warm`), grep(`fn`, "src/**"))
```

prints:

```
estimate.files = 2.3
  0.3                       manual
  2 files                   grep `warm`
  2 files                   grep `fn` in `src/**`
  1.9999999999999998 files  distinct across all queries
```

`crates/mindtape-cli/src/cmd/eval.rs` passes `total - count.manual`, an f64, to `mindtape::count`.

== TO-BE

- 🟢 The distinct note prints the integer union, not a floating-point artifact.
- 🟢 A fractional manual count still prints as authored.

```
estimate.files = 2.3
  0.3        manual
  2 files    grep `warm`
  2 files    grep `fn` in `src/**`
  2 files    distinct across all queries
```

== Changes

- `crates/mindtape-cli/src/cmd/eval.rs`: round the union before formatting.
- `crates/mindtape-cli/src/cmd/eval_tests.rs`: a red test with a fractional manual count.

== Out of scope

- Refusing a fractional manual count.
- Changing the total's number formatting.

== Dogfood

A fractional manual count in this repo's eval output keeps the distinct note readable.
