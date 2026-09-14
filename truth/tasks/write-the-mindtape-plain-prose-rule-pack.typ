#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Write the mindtape-plain-prose rule pack",
  priority: ice(
    impact: 8,
    confidence: 0.6,
    ease: 5,
  ),
  difficulty: 5,
  tags: ("mt", "gating"),
  estimate: (files: 4),
  kind: "feat",
  assignee: "agent",
  status: proposed(2026, 9, 14),
  links: depends-on(
    "ship-named-rule-packs-in-the-binary.typ",
    "add-a-density-content-assert.typ",
  ),
)

== Summary

`ship-named-rule-packs-in-the-binary.typ` delivers the embedding and `include` mechanism with a test-only pack. The pack worth including is the plain-prose one, and its value is entirely in which rules it carries and how tightly they are tuned: the Vale packages behind the density assert deleted their bare-presence variants for false positives.


== Scope

Write the `mindtape-plain-prose` pack as an embedded data file, register it in the pack registry, and pin it with a test that fails a known-slop section and passes a known-clean one:

```toml
[tasks.body.content]
include = ["mindtape-plain-prose"]
```

Calibrate each rule against this repo's own open task bodies, so every bundled rule has an id and passes the current backlog, and use density rules rather than `must-not-match` for the measured tells.

== Dogfood

Add that `include` to this repo's `.mindtape/config.toml`.
