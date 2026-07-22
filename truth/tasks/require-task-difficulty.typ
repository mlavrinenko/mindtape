#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "require task difficulty",
  priority: ice(
    impact: 6,
    confidence: 0.85,
    ease: 7,
  ),
  difficulty: 2,
  tags: ("mt", "docs"),
  status: proposed(2026, 7, 22),
)

== Summary

Require `difficulty` on this repository's own tasks, so the backlog says which model tier and reasoning effort a task wants before anyone starts it. The scale stays generic — a number on the existing 0 to 9 field with the floor's junior / mid / senior / staff anchors — so no vendor or model name enters the artifacts. Mapping a number onto a specific model is the reader's policy, exactly as the difficulty field was designed for.

No feature work: the field, the scale, the anchors and the classification all exist. This is a config change plus a backfill.

The risk worth naming is that difficulty is a prediction, and a prediction is what an agent fills plausibly rather than truthfully. What blunts that is a written rubric: score judgment required, not size. A rubric both a human and an agent can apply the same way is the difference between a routing signal and a decorative number.

== Scope

- Write the rubric into `.mindtape/config.toml` as comments on `[tasks.difficulty]`, anchored on judgment rather than effort. Roughly: 0 to 2 mechanical, the change is fully specified; 3 to 5 local judgment, one module, obvious tests; 6 to 8 cross-cutting design with several defensible approaches; 9 the approach itself is the open question.
- Backfill `difficulty` on every existing task in `truth/tasks`.
- Only then set `required` on `[tasks.difficulty]`. Order matters: flip it first and the whole backlog fails validation at once.
- Add a `difficulty` slot to `.mindtape/templates/task.typ` so `mt add` asks for it rather than leaving the next task to fail validation after it is written.
- Mirror the rubric into the `gated` preset's annotated config, since the routing question is exactly the agent-facing one.
- Tests: the dogfood config parses and validates with `required` set; `mt add` with no explicit difficulty still produces a valid task.
