#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "cover hval_kind in crates/mindtape/src/model.rs",
  priority: ice(
    impact: 7,
    confidence: 0.85,
    ease: 4,
  ),
  difficulty: 6,
  estimate: (files: 1),
  tags: ("swept", "coverage"),
  status: done(2026, 8, 9),
)

== Summary

#raw("hval_kind") in #raw("crates/mindtape/src/model.rs") is 40% covered at cyclomatic complexity 8, inside this arm's band: complexity 5 or above, coverage under 80%.

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
