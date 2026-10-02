#import "../utils/page.typ": page
#import "../utils/site.typ": u

#let meta = (
  route: "/derived-tags/",
  label: "Derived tags",
  title: "Derived tags",
  summary: "How tasks get tags from their folders under a root, without the tags being written to the file.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

A task can inherit #strong[virtual tags] from where it sits under a root — the
root's name plus the folder segments leading to it. They live only in memory and
join the authored `tags` for #link(u("/filters/"))[filtering].

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
enabled = false                # project-wide switch (default): a root opts in below
collect = ["segments"]         # segments and/or breadcrumbs
separator = ":"                # breadcrumb join separator (default)
slugify = true                 # normalise folder names to tag form (default)
breadcrumb-root = false        # fold the root name into the breadcrumb chain (default)

[[tasks.roots]]
path = "tasks"
name = "work"
derive-tags = true             # bool | glob | list — per-root switch
```

Derivation is off by default. A root with its own `derive-tags` uses it —
`true`/`false` wholesale, or a glob (or list of globs) matched against the path
relative to the root; a root without one inherits `enabled`. So turn it on
project-wide, per root, or only for the files a glob matches.
`breadcrumb-root` prepends the root name onto the breadcrumb chain itself — off
by default, since the root name is already emitted as its own standalone tag.

= How they behave

- #strong[Virtual] — never written back to the file.
- Validated strictly against `[tasks.tags] allowed`, the same as authored
  tags, and on open tasks only — a derived tag outside the allow-list fails
  `mt check` on an open task; a closed one keeps whatever its folder derived
  when it was filed.
