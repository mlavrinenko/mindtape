#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Stop counting fenced code blocks in the section word caps",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.95,
    ease: 7,
  ),
  difficulty: 2,
  links: (
    (
      task: "give-the-templates-their-own-command.typ",
      kind: "surfaced-by",
      note: none,
    ),
  ),
  tags: ("found", "mt", "gating", "stateless"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[a word cap measures the prose it is about],
      config-over-code: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
      self-describing: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 11, 21),
)

== AS-IS

+ Fixing the prose in eleven filed tasks so `just docs::vale-tasks` passes
+ 🔴 #raw("crates/mindtape/src/body/content_gate/mod.rs"): `word_count` counts every token in a section, so a fenced code block spends the 120-word cap

```
$ mt check
error: truth/tasks/2026-11/give-the-templates-their-own-command.typ:33: body.content.rules.cap-as-is-at-120-words: cut this section to under 120 words, or split the task in two
note: section `AS-IS` must contain at most 120 words (found 126)
```

The same section splits 116 words total, 89 of them the fenced listing and 27 the prose.

== TO-BE

- 🟢 `max-words` and `min-words` count the section's prose, and a fenced code block doesn't count.
- 🟢 Inline code still counts as a word, since it sits inside a sentence.
- 🟢 The count reuses the fenced-code split the density assertion already runs.

```
$ mt check
note: section `AS-IS` must contain at most 120 words (found 27)
```

== Changes

- `crates/mindtape/src/body/content_gate/mod.rs`: `word_count` drops fenced code before it splits.
- `crates/mindtape/src/body/content_gate/content_gate_density.rs`: `strip_fenced_code` becomes `pub(super)`.
- `crates/mindtape/src/body/content_gate/tests.rs`: a section over its cap in prose fails, and one over only in a fenced block passes.
- `www/content/rules.typ`: the cap counts prose, not a code sample.

== Out of scope

- The cap values. They stay as they are.
- `must-match` and `must-not-match`. A regex over the raw slice still sees code, which is what a shape rule needs.

== Dogfood

The eleven tasks filed in 2026-11 stop spending their AS-IS budget on the output they quote.
