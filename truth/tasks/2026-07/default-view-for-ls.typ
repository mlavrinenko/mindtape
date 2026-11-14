#import "@local/mindtape:0.3.0": *

#show: task.with(
  title: "Default view for ls",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 8,
  ),
  tags: ("cli",),
  links: depends-on("named-views-and-contexts.typ"),
  status: done(2026, 7, 12)[replace default-filter with a default-view],
)

== Summary

Replace `[ls] default-filter` with a `default-view`. `default-filter` names only
a filter, so the default `mt ls` listing cannot set sort or columns. A view
can. Without a filter, `mt ls` resolves the default listing from a view:
`[ls] default-view` (a view name) if set, else `[views.default]` if that view is
declared, else match-all. Drop `default-filter`.

== Scope

- Add `[ls] default-view: Option<String>` (`crates/mindtape/src/config_ls.rs`).
  Validate at config load that it resolves to a declared view.
- Implicit fallback: a view literally named `default` (`[views.default]`) serves
  as the default listing when `default-view` is unset.
- Drop `LsConfig.default_filter` and its use in the `mt ls` command. The
  resolution above replaces it.
- Migrate this repo's `.mindtape/config.toml`: define `[views.default]` with the
  current filter `'!terminal && (miss(start) || start <= now())'` and drop
  `default-filter`. Backlog-config change: task-config commit, no `Refs:`.
- Precedence otherwise unchanged: an explicit CLI filter or `@view` skips the
  default entirely. `mt ls ''` stays match-all.
- Docs: README, `skills/mindtape/SKILL.md`, the `www` config and `cli` pages,
  and CHANGELOG. Refresh the outdatty baseline if a guarded doc changed.
- Tests: `default-view` resolves. The `[views.default]` implicit fallback works.
  An unknown `default-view` errors at load. An explicit filter bypasses the
  default. Match-all applies when neither option exists.

== Non-goals

`mt ls --json` output shape stays a bare task array here. The envelope lands in
`conditional-ls-rules`. Conditional rules, the condition language, the trailer,
and the JSON envelope all belong to that task.
