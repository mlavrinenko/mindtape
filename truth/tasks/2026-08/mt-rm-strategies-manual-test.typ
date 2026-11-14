#import "@local/mindtape:0.3.0": *

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
  status: done(2026, 8, 7)[session run 2026-08-07, every probe passed],
  links: related(
    "mt-rm-should-not-fail-without-a-git-repository.typ",
  )[
    the implementation this session verifies
  ],
)

== Summary

Verify the gitless strategies of `mt rm` (reject, unlink, retire, and the retire
archive move) in a real session outside a git repo, plus the dry-run and batch
cases.

== Outcome

Session run 2026-08-07. One scratch project per strategy under `/tmp`, each
scaffolded fresh with `mt init --no-input` (the `flow` preset, terminal statuses
`done` then `cancelled`). Each project has the strategy appended to the config
and sits outside any git repo, holding `alpha-task` and a `gamma-task` that
carries `depends-on("alpha-task.typ")[needs alpha first]`.

- Default (no `[rm]` `strategy`): exit 1, `cannot remove task(s): 1
  reference(s) from surviving task(s)`, naming the tombstone gap and the two
  escape hatches, then `tasks/gamma-task.typ  links to alpha-task`. Relative
  path, no absolute path in the message. Both files intact.
- `strategy = "unlink"`: the survivor's whole `links:` field dropped (its only
  edge), `alpha-task.typ` deleted, `mt check` green.
- `strategy = "retire"`: `retired 1 task(s) (status → done)`, file stays,
  status flipped `proposed` → `done` (the legend's first terminal), the inbound
  edge still names `alpha-task.typ` and still resolves, `mt check` green.
- `[retire]` `status = "cancelled"`: `status → cancelled` instead.
- `[retire]` `rename = ".archive"`: file moved to `tasks/.archive/alpha-task.typ`
  and the survivor's edge points at `.archive/alpha-task.typ`. `mt check`
  stays green.
- `--dry-run` under both `retire` and `unlink`: printed `strategy: retire` /
  `strategy: unlink`, the would-retire/would-move/would-rewrite plan, and
  `dry run: nothing written`. File checksums unchanged afterwards.
- Batch: `mt rm 'status == "proposed"' -y` removed both tasks at once, with a
  plain `deleted 2 task(s)`, no strategy line, no git error, and `mt check`
  green.

All probes pass. No failures.

== Scope

- Scratch project with a tasks root outside any git repo, one task referencing
  another:
  - default (no `[rm] strategy`): `mt rm` fails and lists the surviving
    referrer and the removed id. The error never shows an absolute path.
  - `[rm] strategy = "unlink"`: edges stripped from the survivor, file
    deleted, `mt check` green.
  - `[rm] strategy = "retire"`: file stays, status flips to the legend's
    first terminal status, inbound edges keep resolving.
  - `[retire] status = "cancelled"`: flips to that status instead.
  - `[retire] rename = ".archive"`: file moves into the sibling folder and
    inbound edges are retargeted to the new path; `mt check` green.
  - `mt rm --dry-run` outside a git repo: prints `strategy: …` and the
    planned edits, and leaves the files alone.
- Same project: `mt rm` both tasks at once (the only referrer is in the
  batch), with plain deletion, no strategy, no git error.
