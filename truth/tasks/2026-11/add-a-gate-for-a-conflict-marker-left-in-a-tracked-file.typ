#import "/truth/prelude/2/lib.typ": *

#show: task.with(
  title: "Add a gate for a conflict marker left in a tracked file",
  kind: "dev",
  priority: ice(
    impact: 6,
    confidence: 0.95,
    ease: 8,
  ),
  difficulty: 2,
  links: (
    (
      task: "retire-mt-explain-in-favor-of-mt-config-show.typ",
      kind: "surfaced-by",
      note: none,
    ),
  ),
  tags: ("found", "gating", "tooling", "stateless"),
  assignee: "agent",
  status: done(2026, 11, 24),
)

== AS-IS

+ Landing the contrib/jq split while resolving its CHANGELOG.md conflict by hand
+ 🔴 #raw("CHANGELOG.md"): a landed commit carried git conflict markers, and every gate passed on that tree

```
$ git show 28cafc4b:CHANGELOG.md | grep -c '^<<<<<<<'
1
$ git log --oneline -1 -- CHANGELOG.md
28cafc4b feat(jq): split the adapters and add the essentials report
```

`just check` reported 44 gates passed on that tree, and on the next commit that still held the markers.

== TO-BE

- 🟢 A tracked text file holding a conflict marker fails a gate, naming the file and the line.
- 🟢 Markers inside a fenced example that documents the shape pass, because a doc may quote one.

```
$ just lint::conflict-markers
error: CHANGELOG.md:12 has a conflict marker `<<<<<<<`
```

== Changes

- `.just/scripts/check-conflict-markers.sh`: read `git ls-files -z` and fail on a line starting with `<<<<<<< `, `=======` or `>>>>>>> `. A file listed in an `ignore` array passes.
- `.just/scripts/_conflict-markers-control.sh`: a clean tree passes and a marker fails by name, as the other scanner gates do.
- `.just/lint.just`: the `conflict-markers` recipe, carrying `[group("gate")]`.
- `Justfile`: one `(mmz "lint::conflict-markers")` arm on `_check-fast`.
- `.mmz/config.yaml`, `.config/outdatty.yaml`: a `script-check-conflict-markers` scope, a gate rule and a group. `just meta::script-scope` fails without them.
- `CHANGELOG.md`: an entry under `[Unreleased]` `Added`.

== Out of scope

- The merge resolution itself. A worktree owner resolves a conflict, and the gate only refuses to land one.
- A general lint over tracked text. This covers the three marker lines git writes.

== Dogfood

This repo lands on `main` from parallel worktrees, so a hand-resolved conflict is the common case, and the marker that reached `28cafc4b` cost a later rebase.

== Goals

+ #Helps(adopter, to: run-checks-at-the-write)[a landed change cannot carry a hand-merged marker].
+ #Neutral(read-tasks-at-a-glance, guide-the-implementer, keep-the-engine-agnostic, adopt-in-one-line, beat-markdown, keep-plain-files, explain-itself, keep-changes-safe, match-docs-to-code).
