#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "split or re-cap CONTRIBUTING.md",
  priority: ice(
    impact: 8,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 3,
  estimate: (files: 2),
  tags: ("swept", "size"),
  status: done(
    2026,
    8,
    10,
  )[sweep deep-dive moved to docs/contributing/signal-sweep.md; 276 -> 231 lines, linecop no longer names it],
)

== Summary

#raw("CONTRIBUTING.md") stands at 289 of its 300-line budget: 11 lines of headroom left, 96.3% of the cap spent.

Prose: the seam is a section boundary, and the question is which sections a reader needs in front of them every time versus which ones they go looking for.

`just ask-refactoring` filed this from `linecop --format json`, and nothing
recorded that it did. The sweep re-reads the signal every run, so this task is
worth exactly as much as the measurement behind it still is.

== Scope

- Pick the remedy, then do it. Either move a section off the always-loaded
  surface, or raise its cap in `.linecop.yaml` with an override that names a new
  sane number and says why. Both are correct answers. Compressing the prose in
  place is not, and `.linecop.yaml` already records what it costs: the 300-line
  overrides exist because a wall mid-edit turned a round of correctness fixes
  into a dozen rewording passes.
- If you move a section, move it somewhere a reader reaches deliberately —
  progressive disclosure into `references/` for a skill, its own page for the
  docs site. What the cap defends is the surface that loads every time, so a
  section still pulled in from a second file has not actually moved.
- If you re-cap, write the reason next to the pattern in `.linecop.yaml`, in the
  style its `SKILL.md` / `AGENTS.md` / `CONTRIBUTING.md` entries already use. A
  written reason is the only thing that suppresses this finding on the next
  sweep — there is no ignore list anywhere else, deliberately.
- Done when `just check-file-size` passes and `linecop --baseline 90` no longer
  names this file.
