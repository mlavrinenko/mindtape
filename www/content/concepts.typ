#import "../utils/page.typ": page
#import "../utils/ui.typ": callout
#import "../utils/site.typ": u

#show: page.with(
  title: "Concepts",
  active: "/concepts/",
  summary: "The model behind MindTape: files as tasks, live evaluation, identity by filename, and the extension seam.",
)

Five ideas explain everything MindTape does.

= A task is a file

One task lives in one `.typ` file. The file applies the task template with a
show rule and declares its fields:

```typ
#show: task.with(title: "…", status: wip(2026, 7, 6), priority: 8)

== Summary
Free-form body follows the metadata.
```

Because the file is Typst, it is real code — see #link(u("/dsl/"))[Task DSL] for
every field. Tasks live wherever you point a root: alongside the code they track,
in a dedicated backlog directory, or both.

= Live evaluation

There is no daemon, no database, and no global index. Every command re-evaluates
the files under your configured roots with the Typst compiler, then applies your
config. The state on disk is the only state.

- Nothing to start, sync, or corrupt.
- Edits by hand, by script, or by `mt` are all equal — the next command sees them.
- The cost is per-command evaluation, not a background process.

= Identity is the filename

A task's id is its filename stem: `tasks/wire-oauth.typ` is `wire-oauth`. Links
between tasks reference that stem through `link("other.typ")`. Rename or move a
file and inbound links stop resolving — `mt check` reports it loudly rather than
silently rotting. Identity you can see in the file tree, not a hidden UUID.

= Core fields and the extension seam

Everything scalar or enum is a generic #strong[core field], constrained by one
config file: the status legend, the numeric priority and difficulty scales,
`kind`, `estimate`, dates, tags, and task links. MindTape validates these and
nothing more.

Tools built on top add their own data through an `x-<ns>` #strong[extension seam]
— any field named `x-<namespace>` is carried through untouched, for that tool to
validate. MindTape stays domain-agnostic; consumers own their own fields.

= Configuration is layered

`.mindtape/config.toml` overlays a shipped defaults floor. Omit a section to
inherit the floor, declare it to override, or set it empty to leave that aspect
unconstrained. Four layers deep-merge in precedence order — embedded floor, XDG
global defaults, ancestor `.mindtape/defaults.toml` files, then the project
config. #link(u("/config/"))[Configuration] covers the merge and every section.

#callout("note")[
  Ready to try it? #link(u("/quickstart/"))[Quickstart] walks install → first task.
  The #link(u("/cli/"))[CLI reference] lists every command.
]
