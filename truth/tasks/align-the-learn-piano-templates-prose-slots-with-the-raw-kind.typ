#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "align the learn-piano template's prose slots with the raw kind",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 2,
  tags: ("docs",),
  estimate: (files: 2),
  assignee: "agent",
  status: done(2026, 9, 9),
)

== Summary

`examples/learn-piano/.mindtape/templates/task.typ` still declares `summary`
and `scope` as `kind: "text"`, the spelling this project's own template just
moved off. Nothing is broken today — the fixture's bodies carry no code
spans, so none is escaped — but the fixture is not private: `www/generate.sh`
copies it verbatim into `www/generated/template.typ` and captures
`mt add --describe` beside it, and `www/content/templates.typ` renders both.
So the page that teaches the `text` / `raw` distinction illustrates it with a
template making the choice the reference argues against
(`skills/mindtape/references/add-slots.md`: "Prefer `raw` when a slot's whole
point is formatted body content").


== Scope

Flip both slots to `kind: "raw"` and regenerate. Two captures move and both
are read by `www/content/templates.typ`:

- `www/generated/template.typ` (`www/generate.sh`, the verbatim copy) — the
  docs' "A template" example.
- `www/generated/describe.txt` (`mt add --describe`) — the two rows change
  from `summary  text  required` to `summary  raw  required`, and likewise
  for `scope`.

The third capture, the written task, is unaffected: `www/generate.sh` fills
`summary` with `"Half-pedal control on sustained chords."`, which carries no
markup either kind would treat differently.

Nothing in `skills/mindtape/evals/evals.json` asserts a slot kind for this
fixture, and `examples/minimal` declares no template at all, so neither
moves.

Should the page say out loud why a body slot is `raw`, next to the example
rather than only in the skill reference an agent reads and a human does not?

= Watch for

`raw` splices verbatim, so a body value that is not valid Typst markup fails
`mt add` outright rather than landing escaped — the intended trade, but it
moves a class of failure into the docs gate, which fills this template on
every run (`www/generate.sh`) and is validated by `just check-examples`.
Does any value that reaches this fixture's slots come from somewhere nobody
reads first?

