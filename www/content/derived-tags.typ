#import "../utils/page.typ": page
#import "../utils/ui.typ": callout
#import "../utils/site.typ": u

#show: page.with(
  title: "Derived tags",
  active: "/derived-tags/",
  summary: "Tasks inherit virtual tags from their location under a root, without those tags ever being written to the file.",
)

A task can inherit #strong[virtual tags] from where it sits under a root — the
root's name plus the folder segments leading to it. Authored `tags` stay
pristine; derived tags live only in memory and join the authored ones for
filtering.

= What gets derived

Given a table-form root and a nested file:

```toml
[[tasks.roots]]
path = "tasks"
name = "work"
derive-tags = true
```

A file at `tasks/backend/auth/login.typ` can derive:

- `segments` — each folder as its own tag: `backend`, `auth`.
- `breadcrumbs` — cumulative paths: `backend`, `backend:auth`.

The root name (the table-form `name`, else the path's last segment) derives too.
Pick what you want under `[tasks.derive-tags] collect`.

= Configuring it

```toml
[tasks.derive-tags]
collect = ["segments"]        # segments and/or breadcrumbs

[[tasks.roots]]
path = "tasks"
name = "work"
derive-tags = true            # bool | glob | list — gates which files derive
```

Per-root `derive-tags` gates which files under that root derive their location:
a bool, a glob, or a list of globs.

= How they behave

- #strong[Virtual] — never written back to the file.
- Held on `Task.derived_tags`; `Task::all_tags` unions them with authored tags,
  so #link(u("/filters/"))[`has_tag(…)`] matches either.
- Validated strictly against `[tasks.tags] allowed`, the same as authored tags —
  a derived tag outside the allow-list fails `mt check`.

#callout("note")[
  Derivation is pure and domain-agnostic: it only reads a file's path under a
  root. Tools built on MindTape get the same behaviour for free.
]
