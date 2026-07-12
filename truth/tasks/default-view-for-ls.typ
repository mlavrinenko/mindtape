#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Default view for ls",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 8,
  ),
  tags: ("cli",),
  links: depends-on("named-views-and-contexts.typ"),
  status: wip(2026, 7, 12)[replace default-filter with a default-view],
)

== Summary

Replace `[ls] default-filter` with a `default-view`. `default-filter` names only
a filter, so the default `mt ls` listing cannot set sort or columns — a view
can. When `mt ls` gets no filter, resolve the default listing from a view:
`[ls] default-view` (a view name) if set, else `[views.default]` if that view is
declared, else match-all. Drop `default-filter`.

== Scope

- Add `[ls] default-view: Option<String>` (`crates/mindtape/src/config_ls.rs`);
  validate at config load that it resolves to a declared view.
- Implicit fallback: a view literally named `default` (`[views.default]`) is used
  as the default listing when `default-view` is unset.
- Drop `LsConfig.default_filter` and its use in the `mt ls` command; the
  resolution above replaces it.
- Migrate this repo's `.mindtape/config.toml`: define `[views.default]` with the
  current filter `'!terminal && (miss(start) || start <= now())'` and drop
  `default-filter`. Backlog-config change — task-config commit, no `Refs:`.
- Precedence otherwise unchanged: an explicit CLI filter or `@view` skips the
  default entirely; `mt ls ''` stays match-all.
- Docs: README, `skills/mindtape/SKILL.md`, www (config + cli), CHANGELOG;
  rebaseline outdatty if a guarded doc changed.
- Tests: `default-view` resolves; `[views.default]` implicit fallback; unknown
  `default-view` errors at load; an explicit filter bypasses the default;
  match-all when neither is set.

== Non-goals

`mt ls --json` output shape stays a bare task array here — the envelope lands in
`conditional-ls-rules`. Conditional rules, the condition language, the trailer,
and the JSON envelope all belong to that task.
