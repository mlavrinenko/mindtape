#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "read a slot value from a file with an at operator",
  priority: ice(
    impact: 3,
    confidence: 0.5,
    ease: 5,
  ),
  difficulty: 4,
  tags: ("cli", "docs"),
  estimate: (files: 5),
  status: proposed(2026, 8, 17),
  links: depends-on(
    "unify-mt-add-input-on-a-key-op-value-token-grammar.typ",
  )[
    a third operator only makes sense once the token grammar exists
  ],
)

== Summary

A third operator in the `key OP value` grammar, reading the value from a file
rather than the command line:

```sh
mt add "Practice arpeggios" -- summary@notes.txt
```

The case it serves is a long `text` slot — a summary or a scope — that is
uncomfortable to type as a shell argument and does not justify writing a
whole `-f` YAML document.

Strictly independent of the grammar task beyond needing it to land first. Do
not bundle them.

== Open question

What the file's bytes ARE, which is the whole design. The value goes into a
`text` slot, so it is markup-escaped like any other caller value — the file
contributes prose, not formatting. That makes `.md` a bad example to reach
for: nothing converts Markdown to Typst, and a reader who passes
`notes.md` would reasonably expect their headings and code spans to survive,
which they would not.

So either the operator is documented as plain-prose-only and the examples all
use `.txt`, or it is not worth having. Markdown-to-Typst conversion is its own
project and is explicitly not in scope here — naming it as the reason this
stays narrow.
