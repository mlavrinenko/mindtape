#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Route the remaining spawns and the dogfood loader off the ambient config",
  priority: ice(
    impact: 7,
    confidence: 1.0,
    ease: 7,
  ),
  difficulty: 2,
  tags: ("tests",),
  estimate: (files: 15),
  status: done(
    2026,
    7,
    28,
  )[every spawn routed; load\_config\_layered\_from added; find\_root test bounded after a planted /tmp/.mindtape broke it],
)

== Summary

The rest of the e2e suite spawns mt inline rather than through a helper, so each of those call sites still read the global defaults.toml of whoever ran the suite. dogfood\_check has the same dependence in process: it validates the repo own backlog through load\_config, which merges that layer, so a stray tasks setting in a developer global file could fail the backlog on their machine alone.

== Scope

- Route every remaining inline spawn through support::mt.
- Expose load\_config\_layered\_from, the layered loader with home and the global config directory injected, so an in-process caller can drop both.
- dogfood\_check loads through it with both dropped.
- Bound the walk in the find\_root test that asserts a bare directory has no project root: find\_root ascends to the filesystem root, so the assertion was about whether the machine keeps a .mindtape above \$TMPDIR.
