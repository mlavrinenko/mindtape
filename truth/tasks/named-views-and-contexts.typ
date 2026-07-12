#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Named views",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 6,
  ),
  tags: ("cli",),
  status: done(2026, 7, 12)[
    phase 1 (views core) shipped; the "context"/auto-activation phases were
    reframed onto `default-view-for-ls` + `conditional-ls-rules`
  ],
)

== Summary

Name and persist queries so recurring `mt ls` filters stop being retyped. A view
is a named bundle in `.mindtape/config.toml` — a filter plus optional sort and
columns — referenced by an `@name` sigil, built from machinery that already
exists (the filter language, the `[ls]` sort/column knobs). Shipped.

The originally-scoped "context" / auto-activation phases were dropped: a stateful
global context is the Taskwarrior/dstask model, and it does not fit MindTape,
which is already project-scoped (one config per repo). The useful kernel — a
default `mt ls` that adapts to ambient project state — is a policy of the default
listing, so it lives on `[ls]`, not on a view. Split to two follow-on tasks
(see below).

== Scope (shipped)

- `[views.<name>]` bundles: `filter` plus optional `sort` / `columns` /
  `description`; validated at config load (each filter parses, columns legal,
  every `in_view` / `@` reference resolves, the reference graph is acyclic).
- `in_view("name")` filter predicate; `@name` desugars to it via a literal-aware
  lexical pass (never rewrites `@` inside a string); the evaluator and evalexpr
  stay untouched.
- `mt ls @view` bundle mode (filter + sort + columns); `@name` inside a compound
  filter is filter-facet-only; `--sort @view`; `--columns @view`; the new
  `--columns` flag.
- `mt views` (list / `<name>` inspect / `--json`).
- `--all` removed — `mt ls ''` lists everything; omitting the filter keeps
  `[ls] default-filter`.

== Reframed out (follow-on tasks)

- `default-view-for-ls`: `[ls] default-filter` names only a filter, so the
  default listing cannot carry sort or columns. Replace it with a `default-view`
  (and an implicit `[views.default]`).
- `conditional-ls-rules`: `[[ls.rules]]` first-match with `import-views`
  composition and a condition mini-language (`on_branch` / `env` / `weekday` /
  `now`), plus an `mt ls` trailer and a JSON envelope. This is the ambient
  auto-activation, expressed as `[ls]` policy rather than a stateful context.
