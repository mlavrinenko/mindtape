#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "validate framework input names as typst identifiers",
  priority: ice(
    impact: 3,
    confidence: 0.7,
    ease: 7,
  ),
  difficulty: 3,
  tags: ("cli",),
  estimate: (files: 2),
  status: proposed(2026, 8, 1),
)

== Summary

A framework input name reaches the DSL twice: as the named argument in a \`framework("name", input: …)\` call and as the \`--set\` key that fills it. Neither path is validated, and a name that is not a Typst identifier — \`\[tasks.priority.frameworks.f.inputs."has space"\]\` — breaks both. Config load accepts it; \`mt add\` then splices \`framework("f", has space: true, )\` and fails only after writing, with \`eval error: expected comma\` and no mention of the config that caused it. A hand-authored template hits the same wall, since no Typst call can name that argument either.

== Scope

- reject at config load: every \`\[tasks.priority.frameworks.\<fw>.inputs.\<name>\]\` key must be a name a Typst named argument can carry (Typst identifier rules, so keywords like \`none\`/\`let\` are out too)
- the message names the framework and the input, the way the sibling \`min \< max\` and \`enum input needs values\` checks do
- \`framework.rs\` is at 93% of the linecop cap and \`framework\_tests.rs\` at 96%, so this needs a module split first — that is most of the cost
- out of scope: unit names in \`\[tasks.estimate\]\`, which reach only a slot prompt, never generated source
