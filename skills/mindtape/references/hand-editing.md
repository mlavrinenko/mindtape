# Authoring or editing a task file by hand

Reference for the mindtape skill. Read this only when a task genuinely needs
what no command covers — chiefly the body. Everything else goes through `mt`;
a hand edit skips what `mt` stamps, slugs, injects, resolves and validates.
`links:` included: `mt link` writes typed edges one at a time or from an
adjacency map, checking the kind, the target and the note rule before it
writes.

A task file is Typst. Edit the fields inside `task.with(...)`:

```typ
#import "@local/mindtape:0.2.0": *
#show: task.with(
  title: "Fix login bug",
  status: wip(2026, 6, 28)[started investigation],  // (name, y, m, d)[note]
  priority: 6,                          // raw int, or a call like ice(...)
  difficulty: 3,
  kind: "bug",                          // enum from config
  assignee: "ada",                      // actor working it, config-bounded
  estimate: (hours: 4),                 // opaque unit dict
  links: depends-on("other-task.typ"),  // typed edges by file stem
  tags: ("backend",),
  due: on(2026, 7, 1),                  // ISO date, or omit
)
```

Rules to respect:
- One task per file; the stem is the identity. Renaming breaks inbound links —
  prefer `mt rm` to retire, which rewrites them to tombstones. Without git,
  `[rm] strategy` decides: `reject` (default, lists the referrers), `unlink`,
  or `retire` (keeps the file, flips the status).
- Status, kind, priority scale, and tags come from `.mindtape/config.toml`.
- Custom/extension fields must be namespaced `x-<ns>` (e.g.
  `x-speconaut: ...`). Every other field name is a declared parameter of
  `task`, so a typo is not kept as an extension — it fails with an
  `unknown task slot` error naming the field, e.g. `titel`.
- Edge helpers take plain stem strings, never `link("a.typ")` — that is
  content, and reads as the trailing note. Passing one (or a second note, a
  named arg, a non-string) is an eval error naming the helper, not a dropped
  edge.
- The body is Typst: a bare `//` starts a line comment and swallows the rest of
  the line, and under `[tasks.comments] policy = "forbid"` `mt check` rejects
  the file. `https://…` is safe (Typst auto-links it); `file://`, other URL
  schemes, and any literal `//` go in backticks.
- After any hand edit, run `mt check`. A wrong-typed value is a violation
  naming its position (`tags[1]`, `estimate.days`, `recur.on[1]`), never a
  silently unset field.