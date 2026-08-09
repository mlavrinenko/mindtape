#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "split or re-cap crates/mindtape/src/template_tests.rs",
  priority: ice(
    impact: 8,
    confidence: 0.7,
    ease: 5,
  ),
  difficulty: 5,
  estimate: (files: 3),
  tags: ("swept", "size", "tests"),
  status: proposed(2026, 8, 10),
)

== Summary

#raw("crates/mindtape/src/template_tests.rs") stands at 488 of its 500-line budget: 12 lines of headroom left, 97.6% of the cap spent.

Test code: ejectest already moved these tests out of their parent once, so the eject recipe has no second move here and any further seam is one you draw by hand.

`just ask-refactoring` filed this from `linecop --format json`, and nothing
recorded that it did. The sweep re-reads the signal every run, so this task is
worth exactly as much as the measurement behind it still is.

== Scope

- Pick the remedy, then do it. Either carve the file along a seam that means
  something, or raise its cap in `.linecop.yaml` with an override that names a
  new sane number and says why. Both are correct answers. A split for its own
  sake is not: the cap is there to push back on scope growth, not on accuracy.
- If you split, the extracted module has to stand on its own and be named for
  what it does. An arbitrary bottom half moved into a `_more.rs` clears linecop
  and misses the point entirely.
- If you re-cap, write the reason next to the pattern in `.linecop.yaml`, in the
  style its `SKILL.md` / `AGENTS.md` / `CONTRIBUTING.md` entries already use. A
  written reason is the only thing that suppresses this finding on the next
  sweep — there is no ignore list anywhere else, deliberately.
- Done when `just check-file-size` passes and `linecop --baseline 90` no longer
  names this file.
