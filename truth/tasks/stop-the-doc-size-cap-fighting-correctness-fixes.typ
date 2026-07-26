#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Stop the doc size cap fighting correctness fixes",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 8,
  ),
  difficulty: 2,
  tags: ("tooling",),
  estimate: (files: 3),
  status: done(
    2026,
    7,
    26,
  )[SKILL.md and AGENTS.md at 300; linecop --baseline 85 wired into both agent docs as the pre-edit check],
)

== Summary

AGENTS.md and skills/mindtape/SKILL.md both sat at exactly 200 of the 200-line Markdown cap, so the next correctness fix hit a wall: roughly 40 lines of genuine skill corrections turned into a dozen compression passes spent on rewording rather than on accuracy. A cap should push back on scope growth, not on getting things right.

== Scope

- Raise \*\*/SKILL.md and \*\*/AGENTS.md to 300 in .linecop.yaml, recording the reasoning in a comment so the number is not re-litigated.
- Point future edits at linecop --baseline 85, which lists what is nearing its cap, in AGENTS.md and the CONTRIBUTING gates table.
- Keep fighting growth: past 300 the answer is progressive disclosure into references/, not a bigger number.
