#import "../utils/page.typ": page
#import "../utils/ui.typ": callout
#import "../utils/site.typ": u

#show: page.with(active: "/import-export/")

MindTape ships one import command and zero export commands, on purpose. Both
sides of that decision are `jq` recipes against the
#link(u("/schema/"))[canonical task-JSON schema] — this page is the cookbook.

= Import

`mt import` reads canonical task-JSON — a file, or stdin by default — and writes
native `.typ` tasks. For each record it:

- maps it onto #emph[this project's] config policy: status legend, priority
  scale, `kind` enum, tag allow-list;
- emits a `task.with(...)` file, lands the batch, and validates it.

There is no `--from`/`--via` source selector: sources are `jq` programs,
upstream of the pipe, documented below. `mt check` still gates the result the
same as any other task.

```sh
mt import                       # canonical JSON from stdin, writes into the first root
mt import export.json           # from a file instead of stdin
mt import --into tasks --to inbox   # nest under tasks/inbox/
mt import --dry-run             # preview: report without writing
mt import --status backlog      # default status (dated today) for a record missing one
mt import --map pending=proposed --map completed=done   # rename statuses before validation
mt import --drop-unmapped       # drop a tag outside [tasks.tags] allowed instead of reporting it
```

#table(
  columns: (auto, 1fr),
  table.header([Flag], [Meaning]),
  [`[FILE|-]`],
  [Positional; canonical task-JSON source. `-` (default) reads stdin.],

  [`--into <DIR>`],
  [Destination directory; `.` (default) is the first configured root.],

  [`--to <SUBDIR>`],
  [Subfolder under `--into` to nest imports into (folder-derived tags).],

  [`--dry-run`], [Report and preview only; writes nothing.],
  [`--status <NAME>`],
  [Default status (dated today) stamped on a record missing one.],

  [`--map <OLD=NEW>`],
  [Status rename, applied before legend validation. Repeatable.],

  [`--drop-unmapped`],
  [Drop a disallowed tag instead of keeping it and reporting a violation.],
)

Every record is reported to stderr: filed or skipped, plus each mapping
decision (`mapped` / `defaulted` / `dropped` / `violated`) and any carried
`x-<ns>` extras.

== Taskwarrior

```sh
task export | jq -f contrib/jq/taskwarrior.jq | mt import
```

`task export` emits Taskwarrior's native JSON; `contrib/jq/taskwarrior.jq`
structurally reshapes it into the canonical schema (config-blind — it never
reads `.mindtape/config.toml`). `mt import` then applies this project's policy
map on top.

== GitHub Issues

```sh
gh issue list --json number,title,body,state,labels,assignees,createdAt,url \
  | jq -f contrib/jq/github.jq \
  | mt import --to github
```

`--to github` nests the batch under a subfolder so it derives a `github` tag.

== todo.txt

todo.txt is a line format, not JSON, so it is not a first-class source — no
parser ships for it. A short `jq -R` (raw-input) recipe covers the common case.

```sh
jq -R -s '
  split("\n") | map(select(length > 0)) | map({
    id: "", file: "", body: "",
    estimate: {}, links: [], derived_tags: [], extras: {},
    title: (
      gsub("\\+\\S+"; "")
      | gsub("^\\([A-Z]\\)\\s*"; "")
      | gsub("^\\d{4}-\\d{2}-\\d{2}\\s*"; "")
      | gsub("\\s+"; " ")
      | rtrimstr(" ")
    ),
    tags: ([scan("\\+\\S+")] | map(ltrimstr("+")))
  })
' todo.txt | mt import --status proposed
```

This strips a `(A)` priority marker and a leading creation date out of the
title and lifts `+project` words into `tags`; `@context` markers are left
verbatim in the title (`--status proposed` assumes the shipped default legend
— point it at whatever your project's legend calls its intake state). Adapt
the regexes to the subset of the format your file uses: a starting recipe, not
a spec.

#callout("note")[
  Computed and empty fields (`id`, `file`, `estimate`, `links`,
  `derived_tags`, `body`, `extras`) can be omitted from a canonical record;
  absent keys default to empty values. `mt import` recomputes `id` from the
  title and ignores `file` and `derived_tags` entirely.
]

#callout("note")[
  All three recipes produce a bare JSON array, one of the two shapes
  #link(u("/schema/"))[`mt import` accepts] (the other being the
  `{"tasks": [...]}` envelope `mt ls --json` emits).
]

= Export

There is no `mt export`. This is a deliberate scope decision, not a gap:

- Every task is already a plain `.typ` file, tracked in git. `git log`,
  `git blame`, and a plain `cp` already work — no export step needed to get
  your data out.
- `mt ls --json` is a lossless projection of every field (see the
  #link(u("/schema/"))[canonical schema]) — anything a foreign export command
  could emit, this already emits, in a stable, documented shape.

A foreign-format export is therefore a #strong[reverse] `jq` program: write it
against `mt ls --json`'s output the same way `contrib/jq/*.jq` write against a
foreign tracker's export. Three starting shapes:

== To todo.txt

```sh
mt ls --json | jq -r '.tasks[] |
  "(\(if .priority | type == "number" and . >= 7 then "A" else "B" end)) " +
  .title + (if (.tags // []) | length > 0 then " " + (.tags | map("+" + .) | join(" ")) else "" end)'
```

== To a CSV / spreadsheet shape

`mt ls --format csv` already covers the common case; for a custom column set,
`mt ls --json` plus `jq -r`'s `@csv` filter shapes anything else:

```sh
mt ls --json | jq -r '["title", "status", "priority", "due"], (.tasks[] |
  [.title, .status.name, .priority, .due]) | @csv'
```

== To a Taskwarrior import skeleton

```sh
mt ls --json | jq '[.tasks[] | {
  description: .title,
  status: (if .status.name == "done" then "completed" else "pending" end),
  tags: .tags,
  due: .due
}]' > taskwarrior-import.json
```

Taskwarrior itself has no bulk-JSON importer built in, but this shape is the
starting point for a project-specific `task import` script or a one-off `jq`
massage into individual `task add` calls.
