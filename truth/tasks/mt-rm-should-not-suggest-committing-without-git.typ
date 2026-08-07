#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "mt rm should not suggest committing without git",
  priority: ice(
    impact: 3,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 1,
  tags: ("cli",),
  estimate: (files: 2),
  status: done(2026, 8, 7)[landed 2026-08-07; just check green on merged HEAD],
  links: related("mt-rm-gitless-manual-test.typ")[
    the session that surfaced this
  ],
)

== Summary

`mt rm` prints `review the changes and commit when satisfied` after every
successful removal, including in a project with no git repository — where there
is nothing to commit, and the hint points at a tool the user deliberately does
not have.

== Scope

- Suppress the commit hint when the project is not inside a git repository;
  keep it verbatim when it is.
- Both emitters carry the line: `rm.rs` and `rm_gitless_report.rs`.
- Regression test asserting the hint is absent from the gitless path and
  present from the git one.
