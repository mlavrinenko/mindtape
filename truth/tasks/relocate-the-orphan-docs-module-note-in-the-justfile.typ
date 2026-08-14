#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "relocate the orphan docs module note in the justfile",
  priority: ice(
    impact: 3,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 2,
  tags: ("tooling",),
  estimate: (files: 2),
  status: proposed(2026, 8, 14),
)

== Summary

A note recording that two recipes moved to `docs.just` sits glued to
`fmt-fixture-check`'s comment block, and the just formatter refuses to let the
two be separated.

== The state

Immediately above `fmt-fixture-check`:

```
# docs::check (build, index with Pagefind, then validate internal links +
# assets) and docs::md-check (the docs-md freshness gate) moved to docs.just —
# truth/tasks/split-the-justfile-into-modules.typ.
# Lock the formatter gates: synthesize a deliberately-mis-formatted TOML,
```

Two unrelated blocks, no blank line, reading as one. The move note belongs
nowhere near `fmt-fixture-check` — it landed there because that is where the
recipes used to sit.

== Why it has not been fixed already

Inserting the blank line does not survive: the just formatter collapses a blank
line between two adjacent comment blocks preceding a recipe, so `just --fmt`
removes it again and `fmt-check` then fails on the un-collapsed version.
Observed while implementing the module-fmt gate.

This is the same formatter behaviour family as the comment truncation in
`truth/tasks/fix-two-corrupted-justfile-comments.typ`: the formatter treats the
comment run before a recipe as one unit.

== Scope

- Decide where the move note actually belongs. Candidates: beside the `mod docs`
  declaration, which is where a reader looks for what the module holds; or
  deleted, since git history and the linked task already record the move and the
  module file's own header explains itself.
- If it stays, it needs a home the formatter will not glue it to something
  unrelated — the `mod docs` site has its own comment block already.
- The commentary relocation task covers the whole file's comment layout. If that
  lands first, fold this in there and close this as done by it rather than
  editing the same lines twice.
- Report the formatter behaviour upstream if it is not already known; a
  formatter that silently merges two comment blocks is a data-loss shape, not a
  style choice.
