#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Add a density content assert",
  kind: "feat",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 5,
  tags: ("mt", "gating"),
  estimate: (files: 5),
  assignee: "agent",
  status: proposed(2026, 9, 14),
)

== Summary

The measured LLM prose tells are densities, not occurrences: the em-dash finding is more than one per paragraph (GPT-4.1 runs 3.28x a human baseline), bold-everything is bold spans per hundred words, and conjunctive-adverb openers are a per-document count. `must-not-match` sees only presence, so banning any of them with it fails every honest use. Three Vale packages (JMill/deslop, Syntaf/vale-llm-slop, srobroek/slopvac) all express these as an occurrence rule with a scope and a max, and all three measured bare-presence variants against real corpora and deleted them for false positives.

== Scope

Add a one-key variant to `ContentAssert`, keeping the existing one-key-table shape:

```toml
assert = { density = { pattern = "—", max = 1, per = "paragraph" } }
```

`per` takes `section`, `paragraph` or `sentence` and defaults to `section`; the assert fails when any one unit carries more than `max` matches. The judgment is the unit split over Typst source: where a paragraph or sentence ends when code blocks, list items and nested subsections sit inside the section. Touches the variant and its deserializer in `crates/mindtape/src/body_content.rs` and `body_content_tests.rs`, the evaluation in `crates/mindtape/src/content_gate.rs` and `content_gate_tests.rs`, and the hand-written schema in `crates/mindtape/src/config_schema.rs` — the same five the max-words variant needed. Red test: a section carrying two em-dashes in one paragraph fails at max 1 and passes at max 2. The generated schema, the config page and the changelog follow in `document-the-density-content-assert.typ`.

== Dogfood

Add a `density` rule to this repo's `[[tasks.body.content.rules]]`, landed in `document-the-density-content-assert.typ`.
