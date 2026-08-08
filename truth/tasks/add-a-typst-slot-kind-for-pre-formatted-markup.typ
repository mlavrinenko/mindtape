#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "add a typst slot kind for pre-formatted markup",
  priority: ice(
    impact: 7,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 4,
  tags: ("cli",),
  estimate: (files: 6),
  links: related(
    "document-that-mt-add-text-slots-escape-markup-instead-of-rendering-it.typ",
  )[the task that documented this and deferred the fix],
  status: done(
    2026,
    8,
    8,
  )[kind: typst splices verbatim; unparseable markup fails before any write, naming the slot],
)

== Summary

A `text` slot escapes every markup character on the way in (`escape_markup` in
`crates/mindtape/src/status_value.rs`, applied by `add.rs::markup_safe` to
every `Text` slot), so a filled body carrying an inline code span lands as a
literal backslash-backtick pair and renders as visible punctuation instead of
code. That is deliberate and safe as a default — a fill value is untrusted
input and splicing it raw would let any `--set` inject arbitrary Typst — but it
leaves no way at all to fill a slot with markup that is genuinely meant as
markup.

The escaping is already documented, and that task explicitly deferred the
opt-out as "a product decision to raise separately". This is that decision. The
cost is no longer hypothetical: filing eleven tasks by hand today produced
eleven corrupted bodies needing a mechanical unescape pass, and the earlier
documentation task had already cleaned eight before them. Every machine-filed
task the quality-signal sweep writes is file paths and code spans, so the
defect scales with exactly the automation this backlog is building.

== Scope

- Add a `typst` slot kind that splices its value verbatim, with no escaping.
  The choice belongs to the template author, not the caller: a template is
  trusted repo content, a `--set` value is not, so `kind: "typst"` is the
  template asserting that this slot's filler is trusted.
- `text` keeps today's escaping and stays the default. A template that says
  nothing must not change behaviour.
- Validate what lands. `mt add` already evaluates the written file
  (`task_edit::verify_written`), so a `typst` slot that produces unparseable
  markup already fails — confirm it fails with an error naming the slot rather
  than a bare Typst diagnostic about a file the author did not write.
- `--describe`/`--describe=json` must report the kind, so a caller can tell
  which slots accept markup before building a payload.
- Document the trust boundary next to the existing "`text` slots are escaped,
  not markup" section in `skills/mindtape/SKILL.md`, and cover the round trip:
  a code span in, a code span out, no backslashes.
- Out of scope: retro-fixing task bodies already on disk. Ten older files still
  carry escaped markup; sweep them separately once the kind exists, so the
  cleanup is verifiable against a working alternative rather than done blind.
