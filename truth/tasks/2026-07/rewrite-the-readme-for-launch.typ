#import "@local/mindtape:0.3.0": *

#show: task.with(
  title: "Rewrite the README for launch",
  priority: ice(
    impact: 7,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 2,
  tags: ("docs",),
  estimate: (files: 2),
  status: done(
    2026,
    7,
    28,
  )[install order, plain-noun architecture, bench table],
)

== Summary

The README leads with the worst install path (git clone, nix develop, cargo build) while the quickstart documents cargo install and prebuilt release binaries. The front page then understates the project to every reader who never reaches the docs site. It also duplicates quickstart content that will drift. The architecture section speaks in project-internal jargon (consumer-blind substrates, labelled-metadata-marker harvesting, parse-plane structure extraction) that means nothing to an outside reader. It frames the absent export command as a correction of the reader rather than a design note.

== Scope

- Install section: cargo install first, prebuilt binary second, from source last. State why each machine needs mt install once.
- Cut what the docs site owns (preset list, init flags) down to a link. The README then stops drifting against quickstart.
- Rewrite the architecture section in plain nouns.
- Soften the no-export line to a factual pointer.
- Say identity is the id field, falling back to the filename stem, matching model.rs rather than the current stem-only claim.
