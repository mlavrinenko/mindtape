#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Replace the task section set with a closed four-section spec",
  kind: "chore",
  priority: ice(
    impact: 9,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 6,
  tags: ("mt", "gating", "docs"),
  estimate: (files: 2),
  assignee: "agent",
  status: proposed(2026, 9, 14),
  links: depends-on(
    "add-a-max-words-content-assert.typ",
    "validate-the-kind-field-against-a-configured-enum.typ",
  ),
)

== Summary

Sixty percent of this backlog carries only Summary and Scope; the other forty percent invented 126 distinct headings, 103 of them used exactly once, including 21 spellings of Why. Nothing closes the set, so every task is free to grow a new section. The four that carry a spec at this grain — the state today, the state after, what was considered and ruled out, and how completion is checked — are the ones no surveyed source omits, while a step-by-step Plan and an Alternatives section are the two the evidence argues hardest against at one-commit scale.

== Scope

Replace Summary and Scope with the four-section set:

```
== AS-IS
== TO-BE
== Ruled out
== Done when
```

across `.mindtape/config.toml` and `.mindtape/templates/task.typ`, close the top-level set with `action = "allowlist"`, and add the Done-when rule that refuses a restatement of the gate already asserted by `[[on.flip]]`. Needs max-words first for the length caps.

Split: this task is the config and the template. So `mt check` stays green on the unmigrated backlog, close the top-level set with `action = "allowlist"` over the union of the old two and the new four, which works because a content rule over an absent section never fires. Dropping Summary and Scope from the set is `close-the-section-allowlist-to-the-four-spec-sections.typ`, after the sweep templates and the open task bodies have moved.
