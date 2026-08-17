#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "rename the typst slot kind to raw",
  priority: ice(
    impact: 7,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 6,
  tags: ("cli", "docs"),
  estimate: (files: 14),
  status: done(
    2026,
    8,
    17,
  )[renamed to raw; unknown kind is now a loud error naming the slot and the valid kinds; sweep template and docs migrated],
  links: related("unify-mt-add-input-on-a-key-op-value-token-grammar.typ")[
    the same milestone: that one fixes the shape of an input, this one fixes
    what an input claims to be
  ]
    + related("accept-typst-list-syntax-in-a-list-slot-set-value.typ")[
      the finding that started this, resolved in the opposite direction
    ],
)

== Summary

Every `mt add` value is data except one. `string`, `int`, `float`, `real`,
`bool`, `date`, `list`, `choice` and `text` all render through the CLI into
Typst; `typst` alone splices the caller's bytes as source.

The name claimed more than the kind delivers. Nothing builds a Typst value out
of a `typst` slot and nothing type-checks one — the value is pasted, and the
only check reads it as top-level markup. Measured against the binary:

```
--set estimate='(files: 2'   =>   written, then fails to load
--set estimate='*emphasis*'  =>   written, then fails to load
```

That check covers a `#slot(…)` in the body, which is what the kind is for. It
cannot see a slot in an argument position: there the value is an expression,
and markup treats parens and `*` as ordinary text.

So the resolution is a rename, not a removal. `raw` names the splice, which is
the actual contract, and the plane gap becomes documented rather than implied
away. Removing the kind outright was the original plan and was wrong: the
sweep's `remedy` slot is a real user, and the per-language markup it carries is
exactly what the kind exists for.

== Scope

- `SlotKind::Typst` → `SlotKind::Raw`, spelled `raw`. `render_typst` →
  `render_raw`. Doc comments in `template.rs`, `template_markup.rs`,
  `status_value.rs` and `add.rs` say what is and is not checked, naming the
  argument-position gap instead of implying full validation.
- A slot declaring an unknown `kind:` becomes an error. It previously fell
  back to `string` silently, so a template still saying `kind: "typst"` would
  fill as a quoted, escaped string with exit 0 and no diagnostic — the rename
  is only safe to take once that fails loudly. Discovery cannot fail (it
  returns a `Vec`), so `SlotSpec` carries the bad spelling in `unknown_kind`
  and `fill` rejects it before coercing any value.
- `.mindtape/templates/sweep-size.typ` and its `contrib/ask-refactoring/size.sh`
  comment move to `raw`. Re-record `sweep-templates`.
- Docs: the slot-kind prose in `www/content/templates.typ`,
  `docs/src/skill.typ` and `skills/mindtape/references/add-slots.md`.
  Re-record `cli-docs`.
- `CHANGELOG.md` — two breaking entries, the rename and the unknown-kind error.

== Done when

`kind: "raw"` splices verbatim, `kind: "typst"` fails naming the slot and
listing the valid kinds with nothing written, and the full gate is green.
