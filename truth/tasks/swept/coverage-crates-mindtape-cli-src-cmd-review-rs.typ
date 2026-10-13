#import "@local/mindtape:0.2.0": *
#import "/truth/lib.typ": *

#show: task.with(
  title: "cover crates/mindtape-cli/src/cmd/review.rs",
  kind: "test",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 2,
  estimate: (src-files: 1),
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
      sound-codebase: helps[unreached lines get a test that fails when they
        break],
      docs-match-code: neutral,
    ),
  ),
  status: proposed(2026, 10, 13),
)

== Problem

+ Run `just cover`.
+ 🔴 #raw("crates/mindtape-cli/src/cmd/review.rs") is 79.1% covered: 14 of 67 instrumented lines are never reached by the suite, against a floor of 88%.

`just ask-refactoring` filed this, and the sweep re-reads the signal every run,
so the task is worth only as much as that measurement still is.

== TO-BE

- 🟢 `just cover` reports this file at or above the floor, or the `just cover`
  recipe excludes it with a reason.

== Changes

- Pick the remedy, then do it: test the unreached paths, or exclude the file
  from scoring with a reason next to its entry in the `just cover` recipe, in
  the style its `main.rs` exclusion already uses.
- If you test, start with the branches that decide something; `just mutants -f`
  on the same file says whether the new tests actually pin behaviour.
- A written exclusion reason is the only thing that suppresses this finding on
  the next sweep; there is no ignore list anywhere else, deliberately.

== Out of scope

- Tests that walk the lines without asserting on them: they move the number and
  buy nothing, which is the failure mode a coverage target invites.

== Dogfood

Not applicable: a sweep finding changes no feature this repo could use.
