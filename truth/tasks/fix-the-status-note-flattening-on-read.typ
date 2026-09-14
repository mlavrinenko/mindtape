#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Fix the status note flattening on read",
  priority: ice(
    impact: 7,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 4,
  tags: ("mt", "cli"),
  estimate: (files: 6),
  assignee: "agent",
  status: done(
    2026,
    9,
    14,
  )[typst-world 0.3.2 published to crates.io (fix f975bbd, keyed off ParbreakElem — the real shape eval-only markup produces, not the hand-built ParElem the first attempt tested against). Bumped here (crates/mindtape/Cargo.toml), cargo test --workspace green (2460+ tests), clippy clean. crates/mindtape-cli/tests/status\_note\_paragraphs\_e2e.rs pins the real mt flip -m / mt ls --json round trip; hoisted the shared status\_note reader into tests/support to keep check-dry green.],
)

== Summary

A multi-line status note round-trips correctly into the artifact: `mt flip done -m` with blank lines and bullets writes a well-formed `done(2026, 9, 14)[...]` block and `mt check` passes. Reading it back through `mt ls --format json` flattens it without separators — "First line of the outcome.Second paragraph explaining the decision. a bullet another". The paragraph break vanishes with no space at all, welding two sentences into one word, and the list markers are dropped. Every consumer of the note gets mangled text, which blocks making the closing note the home for outcome prose.

== Scope

Find where the status notes content block is stringified out of the marker (crates/mindtape/src/marker.rs and the harvest path) and give a paragraph break and a list item a separator. Pin it with a red test asserting a two-paragraph bulleted note survives the flip then ls --format json round trip unbroken.
