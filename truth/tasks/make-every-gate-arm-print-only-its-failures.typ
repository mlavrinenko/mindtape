#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Make every gate arm print only its failures",
  kind: "task",
  priority: ice(
    impact: 7,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 4,
  tags: ("tooling",),
  estimate: (files: 14),
  assignee: "agent",
  status: proposed(2026, 9, 14),
)

== Summary

A green `just check` prints about a hundred lines — mmz hit notes, cargo `Blocking waiting for file lock`, the jscpd table, the outdatty table, gitleaks INF lines, tola progress bars, repeated `generated N files` lines, and a success sentence per fixture script — so a failing arm's one error line lands mid-stream and a reader has to grep for it. Every arm's exit code already carries its verdict; the success prose restates it.

== Scope

- Silence each arm at its source, never by capturing and replaying output: a quiet flag on the tool (`cargo -q`, jscpd, gitleaks `--log-level`, tola, outdatty, linecop, tombi, nextest), and dropped success `echo`s in `.just/scripts/` and `www/generate*.sh`.
- Keep what IS an issue loud: `check-crap`'s SKIPPED notice, the learn-piano example's deliberate warning, every failure message.
- Rewrite the `No chronic` sentence in the `docs/src/` gates deep dive to match.
- Done when a green run prints nothing but the skips and warnings:

```console
$ just check
check-crap: SKIPPED — target/coverage/lcov.info predates …
warning: tasks/repertoire/fur-elise-polish.typ:3: rules.imperative-title: …
```
