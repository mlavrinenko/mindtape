#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Worktree name aliases",
  priority: ice(
    impact: 6,
    confidence: 0.7,
    ease: 6,
  ),
  tags: ("cli",),
  depends-on: (link("worktree-selection-from-anywhere.typ"),),
  status: done(2026, 7, 9),
)

== Summary

A terse `mt @name <cmd>` shortcut that selects a worktree by name instead of
spelling the full `-w <path>`. Builds on the worktree override
(`worktree-selection-from-anywhere`): the leading `@name` is resolved to a path
and spliced into the existing `-w` mechanism, so every command honours it
uniformly with no new discovery seam.

Aliases are app-scoped, not project-scoped — they name *other* projects, so they
live in a standalone app-config file (`dirs::config_dir()/mindtape/worktrees.toml`)
that is read before root resolution, never merged into a project `Config`.

== Config shape

An ordered array of entries, tried top-to-bottom, first match wins:

```toml
# Explicit alias: `mt @sn` → ~/projects/home/speconaut
[[worktrees]]
name = "sn"
path = "~/projects/home/speconaut"

# Base directory: `mt @<name>` → ~/projects/home/<name> when it is a project
[[worktrees]]
base = "~/projects/home"
```

- Alias entries (`name` + `path`) match on the name alone; a broken `path`
  surfaces a loud downstream error rather than silently falling through.
- Base entries (`base`) match `@<token>` when `<base>/<token>` is a MindTape
  project (holds a `.mindtape/`); a base with no matching project is skipped, so
  several bases can be listed and the first that resolves wins.
- `base` (not `root`) names the join, sidestepping the codebase's already
  overloaded "root" (`[[tasks.roots]]`, project root, `resolve_root`, fs root).
- `~` expands to home. A token is a single path segment; `/`, `.`, `..`, and
  empty are rejected so `@../../etc` can't escape.

== Scope

- `crates/mindtape/src/worktrees.rs` — pure `WorktreeRegistry`: parse, classify,
  validate, resolve. Filesystem-injectable (`is_project` predicate, `home`
  passed in) so it is consumer-blind and unit-testable. Hosting it in the domain
  lib keeps it a convergence candidate for speconaut.
- `crates/mindtape-cli/src/worktrees.rs` — IO + argv desugar: locate and read
  the file, resolve a leading `@token`, rewrite it to `-w <path>` before clap
  parses. Only the first post-program arg is sugar; `@name` elsewhere (the view
  sigil, `mt ls @view`) is untouched — position keeps the two `@` uses disjoint.
- `mt worktrees` — list configured aliases and bases (parity with `mt views`).
- Tests: alias/base parse, mixed-entry and duplicate-alias errors, first-match
  order, base skips non-projects, unknown token, token validation, tilde
  expansion, desugar rewrites leading `@` and leaves later `@` alone.
- Docs: README global-flags, `www` cli + agents, the mindtape skill, CHANGELOG.
