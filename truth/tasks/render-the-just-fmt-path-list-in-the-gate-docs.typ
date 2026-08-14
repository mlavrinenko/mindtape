#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "render the just fmt path list in the gate docs",
  priority: ice(
    impact: 5,
    confidence: 0.85,
    ease: 7,
  ),
  difficulty: 3,
  tags: ("tooling",),
  estimate: (files: 6),
  status: proposed(2026, 8, 14),
)

== Summary

`JUST_FMT_PATHS` is described in prose but its contents are not derived from the
just dump, unlike the two path lists rendered beside it.

== Why it is the same bug the milestone exists to fix

`www/gates.jq` folds Justfile variables into `gates.json` through a hardcoded
selection:

```
IN("TOML_FMT_PATHS", "TYPST_FMT_PATHS", "CRAP_EXCLUDES")
```

`docs/src/contributing.typ` renders the first two as bullets straight from that
data. `JUST_FMT_PATHS` arrived after, so the docs describe it in a hand-written
paragraph instead — the exact hand-restated-fact shape the derived gate table
replaced everywhere else. It is one entry away from being derived like its
siblings.

== Scope

- Add `JUST_FMT_PATHS` to the `gates.jq` selection and render it as a third
  bullet from `gate-assignments`, replacing the descriptive sentence with the
  real list.
- A hardcoded four-name list is the same rot one step later. Prefer selecting
  the assignments by a property they share — a naming convention, or whatever
  the dump already distinguishes them by — so the fifth list is carried without
  an edit here. If nothing in the dump supports that, keep the explicit list and
  say in a comment why it has to be maintained.
- Three comments describe the selection as covering "the three" assignments —
  `www/gates.jq`'s header, `www/generate-facts.sh`, and
  `docs/src/contributing.typ` line 17. Each needs to match whatever the
  selection ends up being.
- `www/utils/gate-notes.typ`'s `fmt-check` note never mentioned the just
  formatter at all. Pre-existing gap; fix it here since this task is already in
  that file.
