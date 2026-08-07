#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "emit a plain path list from ls",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 8,
  ),
  difficulty: 2,
  tags: ("cli", "docs"),
  estimate: (files: 5),
  status: done(2026, 8, 8)[landed 2026-08-08; just check green on merged HEAD],
)

== Summary

`mt ls` has no way to emit a bare list of task file paths, so composing it with
any other tool needs `jq`:

```bash
mt ls '' --json | jq -r '.tasks[].file'
```

`--columns file --format csv` comes close but carries a header row and CSV
quoting. A hard `jq` dependency for the most basic composition there is —
handing a selected set of files to another program — is the wrong default for a
tool whose whole premise is that tasks are plain files.

MindTape is the only thing that can evaluate Typst and apply this project's
config policy, so task selection belongs here. Interaction (a picker) and raw
text search belong outside. A path list is the seam between them.

== Scope

- `--format path`: one task file path per line, no header, nothing else.
- `--format path0`: the same, NUL-delimited, for `xargs -0` and paths holding
  whitespace.
- Paths render exactly as the `file` column does today (relative to the project
  root), so the two agree by construction.
- No hyperlinks, no colour, no `applied:` footer on these formats — the output
  is for a program, not a reader.
- Docs: the point is the recipes, so show them.

```bash
rg TODO $(mt ls ready --format path)      # mt narrows by policy, rg searches text
mt ls '' --format path | fzf | xargs mt go
mt ls 'stale(30)' --format path0 | xargs -0 $EDITOR
```

- Ship the fzf recipe rather than fzf: a shell function under `contrib/`, so the
  reader picks their own picker (fzf, sk, fzy, gum). `mt ls -i` already covers
  the built-in case and should not grow into a fuzzy finder — that is a fight
  against a better tool.
