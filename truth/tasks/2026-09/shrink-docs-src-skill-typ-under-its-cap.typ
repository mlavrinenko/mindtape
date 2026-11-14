#import "@local/mindtape:0.3.0": *

#show: task.with(
  title: "shrink docs/src/skill.typ under its cap",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 4,
  ),
  difficulty: 3,
  tags: ("docs",),
  estimate: (files: 2),
  assignee: "agent",
  status: done(2026, 9, 3),
)

== Summary

`docs/src/skill.typ` stands at 317 of its 320-line cap, 3 lines of headroom and 99% spent. `linecop --baseline 85` names it, and `just ask-refactoring` will re-file this finding every run until someone answers it.

It is the source of `skills/mindtape/SKILL.md`, an always-loaded surface. The cap is doing exactly what it set out to do: the next contributor with a paragraph to add hits a wall mid-edit. `truth/tasks/ls-limit-flag.typ` spent the last four lines documenting `-n` and its `truncated:` signal.

== Scope

- Pick the remedy, then do it. Move a section off the always-loaded surface into `skills/mindtape/references/` and link it. Or raise the `**/docs/src/skill.typ` limit in `.linecop.yaml` with an override that sets a new number and says why. Both are correct answers. Compressing prose in place is not the answer: `.linecop.yaml` records what that cost last time.
- If you move a section, move it somewhere a reader reaches deliberately. A section still pulled in from a second file has not moved.
- Done when `linecop --baseline 85` stops naming the file.
