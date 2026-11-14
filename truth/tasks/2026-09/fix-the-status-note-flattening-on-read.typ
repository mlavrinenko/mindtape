#import "@local/mindtape:0.3.0": *

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
  )[typst-world 0.3.2 published to crates.io (fix f975bbd, keyed off ParbreakElem). That is the real shape eval-only markup produces, not the hand-built ParElem the first attempt tested against. Bumped here (crates/mindtape/Cargo.toml). `cargo test --workspace` is green (2460+ tests), and clippy is clean. `crates/mindtape-cli/tests/status_note_paragraphs_e2e.rs` pins the real `mt flip -m` / `mt ls --json` round trip. The change hoists the shared `status_note` reader into tests/support to keep check-dry green.],
)

== Summary

A multi-line status note round-trips correctly into the artifact: `mt flip done -m` with blank lines and bullets writes a well-formed `done(2026, 9, 14)[...]` block and `mt check` passes. Reading it back through `mt ls --format json` flattens it without separators: "First line of the outcome.Second paragraph explaining the decision. a bullet another". The paragraph break vanishes with no space at all, welding two sentences into one word, and the list markers vanish. Every consumer of the note gets mangled text, which blocks making the closing note the home for outcome prose.

== Scope

Find where the status note's content block gets stringified out of the marker (`crates/mindtape/src/marker.rs` and the harvest path). Give a paragraph break and a list item a separator. Pin it with a red test asserting a two-paragraph bulleted note survives the flip and then the `ls --format json` round trip unbroken.
