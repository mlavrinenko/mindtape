#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "accept Typst list syntax in a list slot's --set value",
  priority: ice(
    impact: 5,
    confidence: 0.7,
    ease: 6,
  ),
  difficulty: 3,
  tags: ("cli", "docs"),
  estimate: (files: 5),
  status: cancelled(
    2026,
    8,
    17,
  )[premise inverted: --set stops reading Typst rather than reading more of it, and the \[\]= spelling it took as fixed is retired],
  links: related("rename-the-typst-slot-kind-to-raw.typ")[
    resolved in the opposite direction: `--set` stops reading Typst for every
    slot, rather than starting to read it for one more
  ]
    + related("unify-mt-add-input-on-a-key-op-value-token-grammar.typ")[
      what replaces the `[]=` spelling this task took as fixed
    ],
)

== Summary

`mt add --set tags='("docs", "www")'` is rejected — a list slot takes only the
repeated `--set tags[]=docs` form, or `-f/--values` with a JSON array. But the
sibling slot on the same template already takes Typst source verbatim:
`--set estimate='(files: 2)'` works, because `estimate` is a `typst` slot. So
one `--set` on one command reads Typst for one slot and refuses it for the
next, and the refusal is for the syntax the task file itself is written in.

Both slots end up as Typst in the artifact. The value a user reaches for first
is the one they can see in the file they are filling.

== Scope

- `crates/mindtape/src/template_fill.rs`: a `list` slot accepts a Typst array
  literal as its scalar `--set` value — `("a", "b")`, `("a",)`, `()` — parsed
  as Typst, not split on commas by hand. A value that parses as Typst but is
  not an array of strings stays an error naming the slot.
- The `FillError::Type` message for `SlotKind::List` (currently at
  `template_fill.rs:109`) names the new form first and keeps `[]=` and
  `-f/--values` as the alternatives.
- Tests beside the existing list-slot cases: the three literal shapes above,
  a non-array Typst value, and a bare `name=a,b` — which must STAY an error, or
  the ambiguity this task is closing just moves.
- Docs: `www/content/templates.typ:53` and `docs/src/skill.typ:181` both state
  the `[]=` rule as the only way. Both are hand-written prose about CLI
  behaviour, so both need the new form.

== Open question

Whether `-f/--values` should gain the same tolerance, or stay strictly
JSON/YAML. It is fed by machines rather than typed, so the argument for Typst
syntax there is much weaker — decide before writing the parser, since sharing
one code path across both is what would drag it in.
