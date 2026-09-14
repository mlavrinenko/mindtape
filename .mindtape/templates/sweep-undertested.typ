#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: slot("title", prompt: "cover <function> in <path>"),
  kind: "test",
  priority: ice(
    impact: slot(
      "impact",
      kind: "int",
      prompt: "1-10, rises with how many paths go unexercised",
    ),
    confidence: slot(
      "confidence",
      kind: "float",
      prompt: "0-1, high: llvm-cov and cargo-crap counted, they did not guess",
    ),
    ease: slot(
      "ease",
      kind: "int",
      prompt: "1-10, falls with how many paths a test must construct",
    ),
  ),
  difficulty: slot(
    "difficulty",
    kind: "int",
    prompt: "0-9 on the judgment scale",
  ),
  estimate: slot("estimate", kind: "text", prompt: "(files: N)"),
  assignee: "agent",
  tags: slot("tags", kind: "list", prompt: "swept, coverage"),
)

== Summary

#raw(slot("function", prompt: "the undertested function")) in #raw(slot("path", prompt: "repo-relative path of the file it lives in")) is #slot("percent", kind: "text", prompt: "the function's covered lines as a percentage")% covered at cyclomatic complexity #slot("complexity", kind: "text", prompt: "the function's cyclomatic complexity"), inside this arm's band: complexity #slot("cc_floor", kind: "text", prompt: "the complexity floor the band starts at") or above, coverage under #slot("cov_floor", kind: "text", prompt: "the coverage floor the band ends at")%.

Nothing else in the repo reports this. `just cover` gates the workspace total
only, the file-shaped coverage arm passes this file, and a file-shaped signal
cannot see a function nobody exercises inside a well-covered file.

`just ask-refactoring` filed this from `cargo crap --format json` over the lcov
`just cover` writes — read for the per-function complexity and coverage it
carries, never for the CRAP score it also prints, which this repo's coverage
makes indistinguishable from complexity alone. Nothing recorded that it did. The
sweep re-reads the signal every run, so this task is worth exactly as much as the
measurement behind it still is.

== Scope

- Pick the remedy, then do it. Either exercise the paths nothing reaches, or
  exclude the file from scoring with a reason. Both are correct answers. Splitting
  the function is not one: the complexity floor is how this arm finds a coverage
  gap worth testing, not a claim that the function is too complex — `just crap`
  gates that already, at 30 and against a committed baseline.
- If you test, take the branches that decide something first. Coverage is the
  signal, not the goal — `just mutants -f` on the same file says whether the new
  tests actually pin behaviour, and a test that walks a branch without asserting
  on it moves this number and buys nothing.
- If you exclude, the entry goes in `CRAP_EXCLUDES` in the Justfile with the
  reason beside it, in the style that list's own comment uses. It is file-shaped:
  there is no per-function suppression anywhere, so the reason has to be good
  enough to drop the file from the `just crap` gate too. A written reason is the
  only thing that suppresses this finding on the next sweep — there is no ignore
  list anywhere else, deliberately.
- Done when a re-run — `just cover`, then `just ask-refactoring` — no longer
  names this function.

== Dogfood

Not applicable: a sweep finding changes no feature this repo could use.
