#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Fail fmt-check on any unformatted just module, not only the last",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 1,
  estimate: (src-files: 2),
  tags: ("found", "tooling"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[a gate that prints a diff also fails],
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: helps[just fmt stops rewriting a comment],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 28),
)

== Problem

+ Running `just fmt` while fixing `stop-a-waits-checker-that-runs-mt-from-spawning-itself-without-end`
+ 🔴 #raw("Justfile"): `fmt-check`'s `for f in .just/*.just` loop keeps only the last diff's exit status, so it exits 0 over a diff in `.just/lint.just`, and `just fmt` there deletes the comment line `# lacks.`

```
$ just fmt-check; echo "exit $?"
-# lacks.
+
 [doc("Copy-paste gate: fail on a duplicated Rust block the baseline lacks")]
exit 0
```


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("Justfile").

== TO-BE

- 🟢 `just fmt-check` fails when any `.just/*.just` module differs from `just --dump`.
- 🟢 `just fmt` is a fixpoint on every module and deletes no comment.


== Changes

- `Justfile`: end each loop body in `fmt-check` with `|| exit 1`.
- `.just/lint.just`: lay out the comments around `DRY_FLAGS` and `dry` so `just --fmt` keeps them.


== Rollback

- Revert both files.


== Out of scope

- just's own formatter. Report upstream that `just --fmt` drops a comment line before an attribute, with this file as the repro.


== Dogfood

Every `just check` in this repo runs the fixed gate.
