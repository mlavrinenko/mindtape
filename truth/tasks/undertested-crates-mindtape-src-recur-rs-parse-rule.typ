#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "cover parse_rule in crates/mindtape/src/recur.rs",
  kind: "test",
  priority: ice(
    impact: 3,
    confidence: 0.85,
    ease: 7,
  ),
  difficulty: 2,
  estimate: (files: 1),
  assignee: "agent",
  tags: ("swept", "coverage"),
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
  status: done(
    2026,
    9,
    23,
  )[covered null, non-dict, unknown-kind and missing-raw arms; 68.2->100%],
)

== Problem

+ Run `just cover`, then `just ask-refactoring`.
+ 🔴 #raw("parse_rule") in #raw("crates/mindtape/src/recur.rs") is 68.18% covered at cyclomatic complexity 6, inside this arm's band: complexity 5 or above, coverage under 80%.

Nothing else reports this: `just cover` gates the workspace total only, and a
file-shaped signal cannot see a function nobody exercises inside a well-covered
file. The sweep re-reads the signal every run, so the task is worth only as
much as that measurement still is.

== TO-BE

- 🟢 A re-run of `just cover`, then `just ask-refactoring`, no longer names this
  function.

== Changes

- Pick the remedy, then do it: exercise the paths nothing reaches, or exclude
  the file from scoring in `CRAP_EXCLUDES` in the Justfile, with the reason
  beside it in the style that list's own comment uses.
- If you test, take the branches that decide something first; `just mutants -f`
  on the same file says whether the new tests actually pin behaviour.
- An exclusion is file-shaped, with no per-function suppression, so its reason
  has to be good enough to drop the file from the `just crap` gate too.

== Out of scope

- Splitting the function: `just crap` already gates complexity, at 30 and
  against a committed baseline.
- The CRAP score `cargo crap` also prints: this repo's coverage makes it
  indistinguishable from complexity alone.

== Dogfood

Not applicable: a sweep finding changes no feature this repo could use.
