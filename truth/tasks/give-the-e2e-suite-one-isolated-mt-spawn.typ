#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Give the e2e suite one isolated mt spawn",
  priority: ice(
    impact: 8,
    confidence: 1.0,
    ease: 6,
  ),
  difficulty: 3,
  tags: ("tests",),
  estimate: (files: 9),
  status: done(
    2026,
    7,
    28,
  )[support::mt isolates XDG\_CONFIG\_HOME, HOME, MINDTAPE\_NOW and MINDTAPE\_WORKTREE; global\_config\_e2e pins it with a positive control],
)

== Summary

Every e2e test spawned mt with the ambient environment, and mt layers a global defaults.toml from dirs::config\_dir() under every project config. So the suite was a function of whoever ran it: a two-line hyperlinks setting in a developer global defaults fails hyperlinks\_e2e on a tree with no Rust change in it, reproduced by pointing XDG\_CONFIG\_HOME at a defaults.toml carrying hyperlinks = always. Removing the earlier duplicate unit test closed one symptom, not the hole.

== Scope

- A support::mt spawn that points the global-config lookup at an empty path, caps the ancestor walk at the project, and clears MINDTAPE\_NOW and MINDTAPE\_WORKTREE.
- The same isolation for a caller building its own Command, since the pty tests go through script(1).
- Convert the spawn sites that own a bespoke helper: the pty tests, rm, date\_pin, recur, flip\_hooks, and the rm unit test that cannot reach the tests module.
- A regression test asserting no xdg layer merges, with a positive control that one does when genuinely present.
