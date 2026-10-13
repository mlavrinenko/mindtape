#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "pin compare_field in crates/mindtape/src/sort.rs",
  priority: ice(
    impact: 4,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 4,
  estimate: (files: 1),
  tags: ("swept", "mutants"),
  status: done(
    2026,
    8,
    10,
  )[pin test: bands whose labels sort opposite to their numeric order (low=1/high=6); mutants -f sort.rs: 0 missed],
)

== Summary

#raw("compare_field") in #raw("crates/mindtape/src/sort.rs") survived a mutation at line 110: the suite still passed after #raw("delete match arm \"priority\" in compare_field").

The line is covered — something executes it — but nothing asserts on what it
decides.

`just ask-refactoring` filed this from `cargo mutants`, and nothing recorded
that it did. The sweep re-reads the signal every run, so this task is worth
exactly as much as the measurement behind it still is.

That sweep ran 41 mutants across 1 file(s) (crates/mindtape/src/sort.rs) and left 2 unpinned.

== Scope

- Pick the remedy, then do it. Either write the assertion that kills the mutant,
  or declare it uninteresting in `.cargo/mutants.toml` with a reason. Both are
  correct answers. A test written only to kill a mutant is not: the assertion
  has to be one a reader would want anyway.
- If you assert, assert on the behaviour the mutated line decides, not on the
  literal it was replaced with. A test that pins the constant and nothing else
  moves the survivor without buying a regression net.
- If you exclude, write the reason next to the entry in `.cargo/mutants.toml`. A
  written reason is the only thing that suppresses this finding on the next
  sweep — there is no ignore list anywhere else, deliberately.
- Done when `just mutants -f` on this file reports the mutant as caught.
