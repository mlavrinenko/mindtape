#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "remove the typst slot kind from mt add input",
  priority: ice(
    impact: 7,
    confidence: 0.6,
    ease: 3,
  ),
  difficulty: 7,
  tags: ("cli", "docs"),
  estimate: (files: 12),
  status: proposed(2026, 8, 17),
  links: related("unify-mt-add-input-on-a-key-op-value-token-grammar.typ")[
    the same milestone: that one fixes the shape of an input, this one fixes
    what an input may be
  ]
    + related("accept-typst-list-syntax-in-a-list-slot-set-value.typ")[
      the finding that started this, resolved in the opposite direction
    ],
)

== Summary

Every `mt add` value is data except one. `string`, `int`, `float`, `real`,
`bool`, `date`, `list`, `choice` and `text` all render through the CLI into
Typst; `typst` alone splices the caller's bytes as source. That single
exception is what makes the input surface hard to describe in one sentence,
and it buys less than it looks like.

It also does not validate. `render_typst` (`template_markup.rs:23`) parses on
the MARKUP plane, where parens and `*` are not special, so a `typst` slot
sitting in an argument position is unchecked. Measured against the current
binary:

```
--set estimate='(files: 2'   =>   Ok("#f(estimate: (files: 2")
--set estimate='*emphasis*'  =>   Ok("#f(estimate: *emphasis*)")
```

Both are written to disk. The kind's own doc comment argues the template
author is asserting trust on the caller's behalf, but the author is asserting
it about a value they have never seen, for every future caller — a `--set`, an
env var, a CI job, an agent.

The rule that replaces it: a `mt add` value is data, never source. Typst
markup in a task comes from the template author (a slot default, the prose
between slots) or from a human in `--edit`.

== Blocker

`.mindtape/templates/sweep-size.typ` fills `remedy` as `kind: "typst"`, and
`contrib/ask-refactoring/size.sh:130` writes per-language markup into it that
contains code spans — it names `.linecop.yaml` and `_more.rs` in raw spans. A
`text` slot escapes the backtick, so a straight migration degrades that prose
to literal backticks. `outdatty.yaml`'s `sweep-templates` note argues it exists
because what a remedy IS differs per language, which is real.

Decide before writing any code:

- Split `sweep-size.typ` per language, so the remedy prose is
  template-authored. Correct by the new rule; grows the `sweep-templates`
  drift group from three dependents to five.
- Keep the prose in `size.sh` and drop its code spans. One file, no design
  change, worse prose.
- Keep `SlotKind::Typst` and instead fix only the plane bug — validate on the
  plane the slot lands in. Preserves the sweep untouched, at the price of the
  one-sentence rule.

== Scope

- `crates/mindtape/src/template.rs`: drop the `Typst` variant, its `as_str`
  and `parse` arms. A template declaring `kind: "typst"` must fail with an
  error naming the slot and suggesting `text`, never silently alias.
- `crates/mindtape/src/template_fill.rs`: drop the `SlotKind::Typst` render
  arm. Delete `template_markup.rs` whole.
- `crates/mindtape-cli/src/add.rs` and `crates/mindtape/src/status_value.rs`
  both carry doc comments arguing the `text`/`typst` trust split; both go.
- The blocker's chosen remedy, in `.mindtape/templates/sweep-size.typ` and
  `contrib/ask-refactoring/size.sh`. Re-record `sweep-templates`.
- Tests: `typst_slot_unparseable_markup_names_the_slot` and the `code` slot in
  `add_tests.rs:311` go; add a test that `kind: "typst"` in a template is a
  named error.
- Docs: the `typst`-slot paragraphs in `www/content/templates.typ`,
  `docs/src/skill.typ` and `skills/mindtape/references/add-slots.md`.
  Re-record `cli-surface`.
- `CHANGELOG.md` — breaking change to the template DSL.

== Note

`text` slots keep more than they look like. `escape_markup`
(`status_value.rs:80`) escapes ten characters — backslash, square brackets,
hash, dollar, asterisk, underscore, backtick, at, less-than — and NOT the
hyphen, so a caller's block list still renders as a real Typst list. Only
inline markup — emphasis, code spans, refs — is lost from a caller value.
