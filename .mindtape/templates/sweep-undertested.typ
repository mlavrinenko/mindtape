#import "@local/mindtape:0.2.0": *
#import "/truth/lib.typ": *

#show: task.with(
  title: slot(
    "title",
    prompt: "cover <function> in <path>",
    example: "Cover compare_due in crates/mindtape/src/sort.rs",
  ),
  kind: "test",
  priority: ice(
    impact: slot(
      "impact",
      kind: "int",
      prompt: "1-10, rises with how many paths go unexercised",
      example: 5,
    ),
    confidence: slot(
      "confidence",
      kind: "float",
      prompt: "0-1, high: llvm-cov and cargo-crap counted, they did not guess",
      example: 0.9,
    ),
    ease: slot(
      "ease",
      kind: "int",
      prompt: "1-10, falls with how many paths a test must construct",
      example: 6,
    ),
  ),
  difficulty: slot(
    "difficulty",
    kind: "int",
    prompt: "0-9 on the judgment scale",
    example: 2,
  ),
  estimate: slot(
    "estimate",
    kind: "text",
    prompt: "(src-files: N)",
    example: "(src-files: 2)",
  ),
  assignee: "agent",
  tags: slot("tags", kind: "list", prompt: "swept, coverage", example: (
    "swept",
    "coverage",
  )),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: helps[a complex function nobody exercises gets a test
        that fails when it breaks],
      docs-match-code: neutral,
    ),
  ),
)

== Problem

+ Run `just cover`, then `just ask-refactoring`.
+ 🔴 #raw(slot("function", prompt: "the undertested function", example: "compare_due")) in #raw(slot("path", prompt: "repo-relative path of the file it lives in", example: "crates/mindtape/src/sort.rs")) is #slot("percent", kind: "text", prompt: "the function's covered lines as a percentage", example: "40")% covered at cyclomatic complexity #slot("complexity", kind: "text", prompt: "the function's cyclomatic complexity", example: "9"), inside this arm's band: complexity #slot("cc_floor", kind: "text", prompt: "the complexity floor the band starts at", example: "5") or above, coverage under #slot("cov_floor", kind: "text", prompt: "the coverage floor the band ends at", example: "60")%.

Nothing else reports this: `just cover` gates the workspace total only, and a
file-shaped signal cannot see a function nobody exercises inside a well-covered
file. The sweep re-reads the signal every run, so the task is worth only as
much as that measurement still is.

== TO-BE

- 🟢 A re-run of `just cover`, then `just ask-refactoring`, no longer names this
  function.

== Changes

- Pick the remedy, then do it: exercise the paths nothing reaches, or exclude
  the file from scoring in `CRAP_EXCLUDES` in `.just/crap.just`, with the reason
  beside it.
- If you test, take the branches that decide something first; `just mutants::sweep -f`
  on the same file says whether the new tests actually pin behavior.
- An exclusion is file-shaped, with no per-function suppression, so its reason
  has to be good enough to drop the file from the `just crap::check` gate too.

== Out of scope

- Splitting the function: `just crap::check` already gates complexity, at 30 and
  against a committed baseline.
- The CRAP score `cargo crap` also prints: this repo's coverage makes it
  indistinguishable from complexity alone.

== Dogfood

Not applicable: a sweep finding changes no feature this repo could use.
