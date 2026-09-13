#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Teach task bodies to show the shape not describe it",
  priority: ice(
    impact: 7,
    confidence: 0.8,
    ease: 7,
  ),
  difficulty: 3,
  tags: ("mt", "docs", "gating"),
  estimate: (files: 3),
  assignee: "agent",
  status: proposed(2026, 9, 14),
  links: related(
    "add-a-max-words-content-assert.typ",
  )[a word cap is the stronger forcing function for showing a shape],
)

== Summary

A task that proposes a config or API shape and only describes it in prose
charges the reader a translation step. `check-estimate-drift-when-a-task-closes.typ`
was filed with `{ check = "estimate-drift", gate = true }` buried mid-sentence
when a five-line TOML block states it exactly and is skimmable. Nothing in
`skills/mindtape/SKILL.md` tells an agent to show the shape, and nothing in
`[tasks.body.content]` asks for one, so a body defaults to prose every time.


== Scope

Two halves, and the second needs no new capability — verified against a
scratch project today.

`skills/mindtape/SKILL.md` gets the rule: when a task proposes a shape — a
config key, a function signature, a CLI invocation, a file layout — show it
as a raw block and let the prose carry only what the block cannot say.

The config gets the mechanical half, where a tag says the task is proposing
config surface. The regex is written as a character class rather than a
literal fence so the rule can be quoted inside a task body without closing
the block it sits in:

```toml
[[tasks.body.content.rules]]
path = ["Scope"]
assert = { must-match = "[`]{3}" }
when = 'has_tag("gating") && !terminal'
message = "this task proposes config surface; show the shape in a block"
```

Note that `max-words` is the stronger forcing function of the two: the
cheapest way to satisfy a word cap while describing a shape is to stop
describing it and show it.

