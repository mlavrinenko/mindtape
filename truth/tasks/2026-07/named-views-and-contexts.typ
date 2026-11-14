#import "@local/mindtape:0.3.0": *

#show: task.with(
  title: "Named views",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 6,
  ),
  tags: ("cli",),
  status: done(2026, 7, 12)[
    phase 1 (views core) landed. The "context"/auto-activation phases moved to
    `default-view-for-ls` + `conditional-ls-rules`
  ],
)

== Summary

Name and persist queries so the user stops retyping recurring `mt ls` filters. A view
is a named bundle in `.mindtape/config.toml`: a filter plus optional sort and
columns, referenced by an `@name` sigil. It reuses machinery that already exists
(the filter language, the `[ls]` sort and column knobs). Done.

We dropped the originally-scoped "context" and auto-activation phases. A
stateful global context is the Taskwarrior/dstask model, and it does not fit
this project, which is already project-scoped (one config per repo). The useful
kernel is a default `mt ls` that adapts to ambient project state. That is a
policy of the default listing, so it lives on `[ls]`, not on a view. Split to two
follow-on tasks (see below).

== Scope (delivered)

- `[views.<name>]` bundles: `filter` plus optional `sort` / `columns` /
  `description`. Config load validates them: each filter parses, columns are
  legal, every `in_view` / `@` reference resolves, and the reference graph is
  acyclic.
- `in_view("name")` filter predicate. `@name` expands to it via a literal-aware
  lexical pass (never rewrites `@` inside a string). The evaluator and `evalexpr`
  stay untouched.
- `mt ls @view` bundle mode (filter + sort + columns). `@name` inside a compound
  filter is filter-facet-only. Add `--sort @view`, `--columns @view`, and the new
  `--columns` flag.
- `mt views` (list / `<name>` inspect / `--json`).
- `--all` disappears. `mt ls ''` lists everything, and omitting the filter keeps
  `[ls] default-filter`.

== Moved out (follow-on tasks)

- `default-view-for-ls`: `[ls] default-filter` names only a filter, so the
  default listing cannot carry sort or columns. Replace it with a `default-view`
  (and an implicit `[views.default]`).
- `conditional-ls-rules`: `[[ls.rules]]` first-match with `import-views`
  composition and a condition mini-language (`on_branch` / `env` / `weekday` /
  `now`), plus an `mt ls` trailer and a JSON envelope. This is the ambient
  auto-activation, expressed as `[ls]` policy rather than a stateful context.
