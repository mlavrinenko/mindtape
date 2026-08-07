#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "mt rm gitless manual test",
  priority: ice(
    impact: 4,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 2,
  tags: ("cli", "tests"),
  estimate: (files: 1),
  status: done(2026, 8, 7)[session run 2026-08-07; every probe passed],
  links: related(
    "mt-rm-must-not-require-git-for-a-task-nothing-references.typ",
  )[
    the implementation this session verifies
  ]
    + related("mt-rm-should-not-suggest-committing-without-git.typ")[
      the wart this session surfaced
    ],
)

== Summary

Verify mt rm removes an unreferenced task in a real session outside a git repo, and rewires links when a survivor references the removed task.

== Outcome

Session run 2026-08-07 against scratch projects under `/tmp`, using the release
binary and the installed `@local/mindtape:0.2.0` package (byte-identical to the
in-tree `lib/prelude.typ`). Projects scaffolded with `mt init --no-input` (the
`flow` preset), not a hand-written config.

- Gitless (`git rev-parse` confirms no repo): `mt add buy milk`, `mt wip`,
  `mt done`, `mt rm buy-milk -y` → `deleted 1 task(s)`, exit 0, file gone,
  `mt check` green. No git mentioned in any error.
- Inside a git repo: an unreferenced task still deletes directly, `mt check`
  green.
- Git repo, survivor holding `depends-on("beta-task.typ")`: `mt rm beta-task`
  rewrote the edge onto
  `swh:1:rev:c1f7295...;path=tasks/beta-task.typ` and `mt check` stayed green —
  the tombstone path is unchanged.
- `mt rm --dry-run` outside a repo printed the plan plus `dry run: nothing
  written`; the file was still on disk afterwards.

All four probes pass. One wart, filed separately: `mt rm` prints `review the
changes and commit when satisfied` even in a project with no git repository —
see `mt-rm-should-not-suggest-committing-without-git.typ`.

== Scope

- Scratch project with a tasks root outside any git repo: `mt add` a task, mark
  it done, then `mt rm` it — no git involvement, no error mentioning a
  repository, file gone, `mt check` green.
- Same project inside a git repo: unreferenced task still deletes directly.
- Scratch project where a survivor links to the removed task: `mt rm` rewires
  the edge to the tombstone as before (git path unchanged).
- `mt rm --dry-run` outside a git repo: prints the deletion plan and writes
  nothing.
