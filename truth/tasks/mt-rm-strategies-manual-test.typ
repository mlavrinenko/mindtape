#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "mt rm gitless strategies manual test",
  priority: ice(
    impact: 4,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 2,
  tags: ("cli", "tests"),
  estimate: (files: 1),
  status: accepted(2026, 8, 5)[],
  links: related(
    "mt-rm-should-not-fail-without-a-git-repository.typ",
  )[
    the implementation this session verifies
  ],
)

== Summary

Verify mt rm's gitless strategies — reject, unlink, retire, and the retire
archive move — in a real session outside a git repo, plus the dry-run and
batch cases.

== Scope

- Scratch project with a tasks root outside any git repo, one task referencing
  another:
  - default (no `[rm] strategy`): `mt rm` fails and lists the surviving
    referrer and the removed id; the error never shows an absolute path.
  - `[rm] strategy = "unlink"`: edges stripped from the survivor, file
    deleted, `mt check` green.
  - `[rm] strategy = "retire"`: file stays, status flips to the legend's
    first terminal status, inbound edges keep resolving.
  - `[retire] status = "cancelled"`: flips to that status instead.
  - `[retire] rename = ".archive"`: file moves into the sibling folder and
    inbound edges are retargeted to the new path; `mt check` green.
  - `mt rm --dry-run` outside a git repo: prints `strategy: …` and the
    planned edits, writes nothing.
- Same project: `mt rm` both tasks at once (the only referrer is in the
  batch) — plain deletion, no strategy, no git error.
