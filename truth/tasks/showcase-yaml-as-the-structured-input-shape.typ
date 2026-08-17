#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "showcase YAML as the structured input shape",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 2,
  tags: ("docs",),
  estimate: (files: 4),
  status: done(
    2026,
    8,
    17,
  )[templates.typ, skill.typ and add-slots.md now lead -f with a YAML heredoc and a block scalar],
  links: related("unify-mt-add-input-on-a-key-op-value-token-grammar.typ")[
    the flag half of the same interface
  ],
)

== Summary

`-f` already takes YAML — `add_values.rs:197` sniffs stdin and any unknown
extension, trying JSON only on a leading `{`/`[` and falling through to YAML
otherwise, and YAML is a JSON superset besides. The docs lead with JSON
anyway, so the compact shape is the one nobody sees.

JSON is the worse shape for exactly the field that matters most. A task's
`summary` is prose: it carries newlines, quotes and punctuation, all of which
JSON makes the author escape. YAML block scalars carry it verbatim.

```sh
mt add -f - <<'YAML'
title: Practice arpeggios
estimate.files: 2
tags: [docs, www]
summary: |
  Work the 7th chords hands-separate, then together at 60bpm.
YAML
```

This is also the shape to point an agent at. A structured channel with no
shell quoting and no operator vocabulary beats any flag spelling for a caller
that is generating input rather than typing it.

== Scope

- `www/content/templates.typ`: lead the `-f` section with the YAML heredoc
  above, demote JSON to a one-line mention that the same channel takes it.
- `docs/src/skill.typ`: the agent-facing page should say plainly that `-f -`
  with YAML is the preferred shape for an agent, and why (no escaping, block
  scalars for prose).
- `skills/mindtape/references/add-slots.md` carries the same JSON-first
  framing; move it with the others.
- All three are `cli-surface` dependents — re-record with
  `just outdatty-update cli-surface`.
- No Rust changes. The parser already does this; only the documentation
  chooses the wrong example.
