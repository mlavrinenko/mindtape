#import "@local/mindtape:0.3.0": *

#show: task.with(
  title: "lint the prose inside a section not only the headings",
  priority: ice(
    impact: 7,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 5,
  tags: ("gating", "docs"),
  estimate: (files: 9),
  assignee: "agent",
  status: done(
    2026,
    9,
    1,
  )[content-rule family implemented: non-empty, min-words, must-match/must-not-match over section source. Reused HeaderPathSegment paths plus when/message. Dogfood rules configured in .mindtape/config.toml. `just check` green.],
)

== Summary

Every `[[tasks.body.headers.rules]]` action operates on the heading tree and
nothing else (`require`, `forbid`, `unique`, `order`, `allowlist` in
`crates/mindtape/src/body_headers.rs:68`). A project can say a task must have
exactly one `= Summary`, and that `= Scope` follows it. It cannot say
anything at all about what sits under those headings.

That is where task quality actually lives. A task with a `= Summary` heading
over one vague sentence passes `mt check` today, and a section titled
"Watch for" without a risk or a question passes too. The heading
is the cheap half to get right and the only half the tool checks.

Concretely wanted: a section that must contain a `?` (an open question
section that states no question is not doing its job). A section must
be non-empty. Its prose must match or must not match a regex. A minimum
length catches a placeholder. All four form one rule
family over section content, gated by the same `when` filter and carrying
the same `message` override the heading rules already have.


== Scope

- A `content` rule family beside the heading rules selects a section by
  the existing `path`, so the `HeaderPathSegment` regex matching carries over
  rather than a new matcher. It asserts over the prose beneath.
- Assertions worth having: \`must-match\` / \`must-not-match\` (regex),
  \`min-words\`, \`non-empty\`. Each carries \`message\`, like every existing rule.
- The section's content is everything under the heading up to the next
  heading of the same or higher level, with nested subsection prose
  included. Decide whether code blocks and raw spans count toward
  `min-words`: a scope section that is all `#raw` is a real shape and
  should not read as empty.
- \`when\` gating, so a rule can apply to \`kind == "bug"\` and not to an
  umbrella, exactly as the heading rules already do.
- Docs: \`docs/src/skill.typ:267\` lists what \`mt check\` validates and would
  grow this. The skill's own "Validating" section names "body headers",
  which stops being the whole truth.

Already supported, and worth documenting rather than building: a
section-title vocabulary is \`allowlist\` plus a regex segment, which the
config reference does not show anywhere obvious:

\`\`\`toml
\[\[tasks.body.headers.rules\]\]
action = "allowlist"
allowed = \[{ match = "^(Summary|Scope|Watch for|Decisions)\$" }\]
\`\`\`

A `kimmable` maintainer asked for exactly this feature without knowing it
existed, which is a documentation finding on its own.

