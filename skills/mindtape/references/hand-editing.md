# Editing a task file by hand

Reference for the mindtape skill. Read this only when a task needs what no
command writes: the body, a framework priority, the `estimate` dict or an
`x-<ns>` slot. Change everything else with `mt` commands. A hand edit skips
what `mt` stamps, slugs, injects, resolves and validates.

| To change | Run |
|-----------|-----|
| `links:` | `mt link`, one typed edge at a time or from an adjacency map. It checks the kind, the target and the note rule before it writes. |
| `title`, `priority` (the raw number), `difficulty`, `kind`, `assignee`, `tags`, `due`, `start` | `mt set <task> <field>=<value>`. It validates before it writes. |
| Status | `mt <status> <task>` |

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
  links: depends-on("other-task.typ"),  // path from this file to the target
  tags: ("backend",),
  due: on(2026, 7, 1),                  // ISO date, or omit
)
```

Rules:

- Keep one task per file. The stem is the identity, so a rename breaks inbound
  links. To retire a task, run `mt rm`, which rewrites them to tombstones.
  Without git, `[rm] strategy` decides: `reject` (the default, lists the
  referrers), `unlink`, or `retire` (keeps the file, flips the status).
- Take statuses, kinds, the priority scale and tags from
  `.mindtape/config.toml`.
- Name a custom field `x-<ns>`, such as `x-speconaut: ...`. Every other field
  name is a declared parameter of `task`, so a typo fails instead of becoming
  an extension: a field written `titel:` fails with
  `unexpected argument: titel`.
- Pass an edge helper plain path strings, never `link("a.typ")`. That is
  content and reads as the trailing note. A `link(...)`, a second note, a named
  argument or a non-string is an eval error naming the helper, never a dropped
  edge.
- Write the body as Typst. A bare `//` starts a line comment and swallows the
  rest of the line, and under `[tasks.comments] policy = "forbid"` `mt check`
  rejects the file. `https://…` is safe, because Typst links it. Put
  `file://`, other URL schemes and any literal `//` in backticks.
- After any hand edit, run `mt check`. It reports a wrong-typed value as a
  violation naming its position (`tags[1]`, `estimate.days`, `recur.on[1]`),
  never as an unset field. It names a file that fails to evaluate with its
  line and column: `error: tasks/x.typ:5:21: expected comma`.
