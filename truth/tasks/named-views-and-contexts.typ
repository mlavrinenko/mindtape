#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Named views and contexts",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 6,
  ),
  tags: ("cli",),
  status: wip(2026, 7, 9)[start phase 1: views core],
)

== Summary

Name and persist queries so recurring `mt ls` filters stop being retyped. A view
is a named bundle declared in `.mindtape/config.toml` — a filter plus optional
sort and column set — built from machinery that already exists (the filter
language, the `[ls]` sort/column knobs). Three orthogonal concerns fall out; keep
them separate:

- Views: the named bundles, referenced with an `@name` sigil.
- Auto-activation (the "context"): pick a default view from project/environment
  state. Stateless — the environment is the state, no on-disk active pointer.
- Worktree selection: run `mt` from anywhere against a chosen worktree. Fully
  orthogonal, split out to the `worktree-selection-from-anywhere` task.

This replaces the earlier Taskwarrior/dstask stateful-context marker: a context
is just a view that auto-activates, and "which project" is a worktree concern,
not a context one.

== Scope

=== Grammar (the `@` sigil)

A view is named by `@name` wherever a value can reference one:

- Filter position, bare `@name` as the whole argument, is bundle mode: apply the
  view's filter, sort, and columns at once (`mt ls @broken`). Facet flags still
  override (`mt ls @broken --sort @summer` keeps broken's filter and columns,
  takes summer's sort).
- Filter position, `@name` inside an expression, is the filter facet only, equal
  to `in_view("name")`; sort and columns are ignored — there is no single view to
  attach them to (`mt ls '@summer || @broken'`, `mt ls '@summer && !@broken'`).
- `--sort @name` splices the view's sort chain (composes with plain `--sort`
  specs; the flag is repeatable).
- `--columns @name` selects the view's column set.

`in_view("name")` is a new boolean filter predicate. `@name` desugars to it via a
literal-aware lexical pass at parse time (never rewrites `@` inside a string
literal); the evaluator and evalexpr stay untouched, seeing only `in_view(...)`.
Bare `@name` subsumes a `--view` flag, so there is no `--view`.

=== Config

```toml
[views.summer]
filter = 'has_tag("summer") && !terminal'
sort = ["priority:desc", "due:asc"]              # optional
columns = ["#", "status", "priority", "title"]   # optional, subset of ls columns
description = "Summer push"                       # optional, shown by mt views

[views.broken]
filter = 'blocked && stale(30)'
```

Validated at config load, alongside `[ls] default-sort`: each filter parses; sort
specs are valid; columns are a subset of the ls column vocabulary; every `in_view`
or `@` reference resolves; the view-reference graph is acyclic.

=== `--columns` flag

New `mt ls --columns <spec>`, spec a comma list of ls column names or `@view`.
Overrides `[ls] columns-order` for the run.

=== mt views

New subcommand. `mt views` lists views (name, filter, which facets are set);
`mt views <name>` shows the resolved bundle; `--json` for scripting. Replaces the
rejected `mt ls --list-views`.

=== Precedence and `--all` removal

Drop `mt ls --all`. Omitted filter uses `[ls] default-filter`; any explicit
filter, including empty `''`, is used verbatim and ignores the default
(`mt ls ''` lists everything). An empty or whitespace filter is a match-all
sentinel handled before evalexpr, which would otherwise reject the empty string.

=== Auto-activation (context, follow-on)

A view auto-activates from environment/project state, ANDing its filter into the
default `mt ls` and seeding `mt add` tags. Tiers, cheapest first:

+ Env var `MINDTAPE_CONTEXT=<view>`. direnv binds it per directory or branch, so
  git-aware auto-context needs no git introspection inside `mt`.
+ Manual pin `mt context <name>` / `mt context none`, stored in XDG state keyed
  by project root (per-user, untracked).
+ `activate-when = '<expr>'` on a view — a second evaluator over the same
  evalexpr parser with environment leaves (`git_branch()`, `env("X")`, `cwd()`,
  `glob(x, pat)`). Build only if tiers 1-2 fall short.

=== Phasing

+ Views core: `[views]` config with load-time validation and cycle check, the
  `in_view` predicate, the `@` desugar, `--columns`, `mt views`, drop `--all`.
+ Auto-activation tiers 1-2 (env var, manual pin, add-tags).
+ `activate-when` expressions, if warranted.

=== Tests

View resolution and each `@` position (bundle vs expression); facet-flag override
precedence; `in_view` composition and cycle rejection; unknown-view and
unknown-column errors; empty-filter match-all; `mt views` output.
