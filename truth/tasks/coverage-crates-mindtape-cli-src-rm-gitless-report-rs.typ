#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "cover crates/mindtape-cli/src/rm_gitless_report.rs",
  priority: ice(
    impact: 3,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 2,
  estimate: (files: 1),
  tags: ("swept", "coverage"),
  status: proposed(2026, 8, 8),
)

== Summary

#raw("crates/mindtape-cli/src/rm_gitless_report.rs") is 84.27% covered: 14 of 89 instrumented lines are never reached by the suite, against a floor of 88%.

`just ask-refactoring` filed this from the lcov `just cover` writes, and nothing
recorded that it did. The sweep re-reads the signal every run, so this task is
worth exactly as much as the measurement behind it still is.

== Scope

- Pick the remedy, then do it. Either test the unreached paths, or exclude the
  file from scoring with a reason. Both are correct answers. A test that walks
  the lines without asserting on them is not: it moves the number and buys
  nothing, which is the failure mode a coverage target invites.
- If you test, start with the branches that decide something. Coverage is the
  signal, not the goal — `just mutants -f` on the same file says whether the new
  tests actually pin behaviour.
- If you exclude, write the reason next to the entry in the `just cover` recipe,
  in the style its `main.rs` and `recur_rrule.rs` exclusions already use. A
  written reason is the only thing that suppresses this finding on the next
  sweep — there is no ignore list anywhere else, deliberately.
- Done when `just cover` reports this file at or above the floor.
