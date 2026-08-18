#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "read a slot value from a file with an at operator",
  priority: ice(
    impact: 3,
    confidence: 0.5,
    ease: 5,
  ),
  difficulty: 4,
  assignee: "agent",
  tags: ("cli", "docs"),
  estimate: (files: 5),
  status: done(
    2026,
    8,
    18,
  )[key\@path landed in both carriers; the open question is answered yes — every kind reads the file, since the operator carries and the slot kind decides. Trailing newlines stripped, so it is a drop-in for \$(cat path). just check green, mutants-diff clean.],
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

== What the file's bytes are

The slot's kind decides, and that is the whole design — there is no conversion
step anywhere in this task.

- Into a `text` slot the bytes are markup-escaped like any other caller value.
  The file contributes prose, not formatting, so the examples must use `.txt`.
  `.md` is a trap: nothing converts Markdown to Typst, and a reader passing
  `notes.md` would reasonably expect headings and code spans to survive, which
  they would not. Markdown-to-Typst conversion is its own project and is
  explicitly out of scope — named here as the reason this stays narrow.
- Into a `raw` slot the bytes are Typst, spliced verbatim, and no conversion
  is even conceptually needed. This is the case that makes the operator worth
  building:

  ```sh
  mt add -- remedy@remedy.typ
  ```

  It buys a workflow the flag form cannot: write the `.typ`, check it, then
  file the task. Today the same markup has to survive a shell argument, which
  is how `contrib/ask-refactoring/size.sh` ends up holding a heredoc per
  language.

== Open question

Whether the operator reads the file at all for kinds that are neither `text`
nor `raw`. A `date` or `int` slot fed from a file is legal but pointless, and
allowing it costs nothing except a wider surface to document. Decide before
writing the parser.
