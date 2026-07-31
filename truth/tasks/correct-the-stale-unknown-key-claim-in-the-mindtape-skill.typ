#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Correct the stale unknown-key claim in the mindtape skill",
  priority: ice(
    impact: 6,
    confidence: 0.95,
    ease: 9,
  ),
  difficulty: 1,
  tags: ("docs",),
  estimate: (files: 1),
  status: proposed(2026, 7, 31),
)

== Summary

skills/mindtape/SKILL.md still teaches the pre-fix behaviour: the section heading reads "--set keys are slot names, and unknown ones are dropped silently" and the body claims \`mt add fix bug --set priority=3\` "reports success while writing the template's default priority". reject-unknown-mt-add-set-keys landed that hard error on 2026-07-26 — the real output is \`Error: unknown --set/-f key(s): priority; known slots: title\` (verified against the current binary). An agent reading the skill is told a clean exit proves nothing and to defensively --print, which is now wasted work, and the "When the tool fights you" tail ("Silence is not success for fill values") points back at the same stale note.

== Scope

- Rewrite the section heading and body to state the hard error, quoting the real message shape (\`unknown --set/-f key(s): \<keys>; known slots: \<names>\`).
- Fix the "Silence is not success for fill values" pointer in the closing section, which only exists to reference that stale note.
- Keep the surrounding, still-correct guidance: slot names are not field names (framework leaves), and a \`list\` slot needs \`\[\]=\`.
- The skill's \`mt add\` flag list documents neither \`-f/--values\` nor its JSON/YAML dispatch — add a line while in there.
- SKILL.md is an outdatty dependent of the \`cli-docs\` group and is capped at 300 lines by linecop; check \`linecop --baseline 85\` before adding.
