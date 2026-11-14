#import "@local/mindtape:0.3.0": *

#show: task.with(
  title: "mt rm must not require git for a task nothing references",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 4,
  tags: ("cli", "tests"),
  estimate: (files: 3),
  links: related(
    "mt-rm-should-not-fail-without-a-git-repository.typ",
  )[sibling fix: degrade gracefully when no git repo exists]
    + related("mt-rm-gitless-manual-test.typ")[
      session verifies the gitless deletion end to end
    ],
  status: done(2026, 8, 5)[gitless deletion path landed and verified],
)

== Summary

The reported gitless failure hit a task with no inbound references. Such a task
has nothing to rewire, so `mt rm` can do without git entirely: no `toplevel`
discovery, no tombstone mint, no committed-clean guard. Yet the git requirement
is unconditional: `run` calls `git::toplevel` and plans every
removal through `typst_cleanup::plan`, which mints a tombstone per eligible
task (`crates/mindtape-cli/src/rm.rs:110-118`). A task with zero inbound
references should delete directly, with zero git involvement.

== Scope

- Before touching git, decide whether a tombstone is actually needed. Scan the
  survivors for inbound references to the eligible ids (the parse-plane
  `link_target_spans` + `TaskLinks` policy `rm` already uses). That scan is
  cheap, with no eval and no git.
- When no survivor references an eligible task: skip `git::toplevel` and the
  committed-clean guard, delete the task files directly.
- Keep the current git + tombstone path when a reference does exist and the
  repository is discoverable.
- Test: a unit test in `crates/mindtape-cli/src/rm_tests.rs` and/or an e2e in
  `crates/mindtape-cli/tests/rm_e2e.rs` removing an unreferenced task both
  inside and outside a git repo.
