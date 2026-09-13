#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Add a density content assert",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 5,
  tags: ("mt", "gating"),
  estimate: (files: 6),
  assignee: "agent",
  status: proposed(2026, 9, 14),
)

== Summary

The measured LLM prose tells are densities, not occurrences: the em-dash finding is more than one per paragraph (GPT-4.1 runs 3.28x a human baseline), bold-everything is bold spans per hundred words, and conjunctive-adverb openers are a per-document count. `must-not-match` sees only presence, so banning any of them with it fails every honest use. Three Vale packages (JMill/deslop, Syntaf/vale-llm-slop, srobroek/slopvac) all express these as an occurrence rule with a scope and a max, and all three measured bare-presence variants against real corpora and deleted them for false positives.

== Scope

Add a one-key `{ density = { pattern = "...", max = N, per = "section" } }` variant to ContentAssert, keeping the existing one-key-table shape, with `per` taking section, paragraph or sentence and defaulting to section. Red test: a section carrying two em-dashes in one paragraph fails at max 1 and passes at max 2.
