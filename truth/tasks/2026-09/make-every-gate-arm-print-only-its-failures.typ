#import "@local/mindtape:0.3.0": *

#show: task.with(
  title: "Make every gate arm print only its failures",
  kind: "dev",
  priority: ice(
    impact: 7,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 4,
  tags: ("tooling",),
  estimate: (files: 14),
  assignee: "agent",
  status: done(2026, 9, 14),
)

== Summary

A green `just check` prints about a hundred lines. The output includes mmz hit notes, cargo `Blocking waiting for file lock`, the jscpd table, the outdatty table, and gitleaks info lines. It also prints tola progress bars, repeated `generated N files` lines, and a success sentence per fixture script. A failing arm's one error line lands mid-stream, and a reader has to grep for it. Every arm's exit code already carries its verdict, and the success prose restates it.

== Scope

- Silence each arm at its source, never by capturing and replaying output. Use a quiet flag on the tool (`cargo -q`, jscpd, gitleaks `--log-level`, tola, outdatty, linecop, tombi, nextest), and drop success `echo`s in `.just/scripts/` and `www/generate*.sh`.
- Keep every real issue loud: `check-crap`'s skipped notice, the learn-piano example's deliberate warning, every failure message.
- Rewrite the `No chronic` sentence in the `docs/src/` gates deep dive to match.
- Done when a green run prints nothing but the skips and warnings:

```console
$ just check
check-crap: SKIPPED — target/coverage/lcov.info predates …
warning: tasks/repertoire/fur-elise-polish.typ:3: rules.imperative-title: …
```
