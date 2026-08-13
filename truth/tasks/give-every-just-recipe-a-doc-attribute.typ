#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "give every just recipe a doc attribute",
  priority: ice(
    impact: 7,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 2,
  tags: ("tooling", "docs"),
  estimate: (files: 1),
  status: wip(2026, 8, 14),
)

== Summary

Move every recipe's one-line description into a `[doc("...")]` attribute, so the JSON dump's `doc` field carries a real description instead of the tail fragment of a rationale comment block.

== Why

just takes the *last comment line* before a recipe as its doc. This Justfile carries long rationale blocks, so 22 of 45 recipes advertise a sentence fragment — or nothing — in `just --list`, the command CONTRIBUTING.md tells contributors to run:

- `check` → "its non-zero exit out of \`parallel\` unchanged."
- `check-doc-facts` → "docs-gen\` — or a release build — having run first."
- `check-examples` → "could have run against it and nothing said so."
- `crap` → null

This is also why `docs/src/contributing.typ` hand-writes its whole `gate-tools` dict: `www/generate-facts.sh` already derives `{name, doc}` per gate from `just --dump --dump-format json`, and the doc half was unusable, so the table threw it away and restated everything by hand.

== Scope

- `[doc("...")]` on every recipe, private ones included. Verified on just 1.43.1: the attribute overrides the comment-derived doc and lands in the dump's `doc` field.
- Long rationale comment blocks stay exactly where they are — they are for a reader of the Justfile, not for a list.
- The blank line some blocks carry before their final comment line is a fossil of working around this; remove it once the doc attribute owns the description.
- No change to `www/generate-facts.sh`: `.doc` simply becomes true.
- Justfile is an `outdatty` `dev-docs` source, so the dependents need re-confirming and `outdatty.lock` recommitting.
