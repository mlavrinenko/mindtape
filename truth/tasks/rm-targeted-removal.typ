#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Rename cleanup to rm; accept a task ref or filter",
  status: done(2026, 6, 20),
  priority: ice(impact: 6, confidence: 0.8, ease: 5),
  estimate: (hours: 4),
  relates-to: (link("reviewed-reaffirmation-field.typ"),),
  tags: ("cli", "substrate"),
)

== Summary

Rename the `cleanup` command to `rm` and let it accept one optional positional
argument that selects what to retire. The argument is either a task reference
(slug, relative/absolute path, or a `list` handle like `1v`) or a filter
expression. With no argument, `rm` falls back to a configurable default filter
(built-in: `terminal`), preserving the old "sweep every terminal-status task"
behaviour. The retire mechanism is unchanged: mint a `swh:1:rev` tombstone per
removed file, rewrite inbound `link()` references onto it, delete the file, do
not commit.

== Scope

- CLI: drop the `cleanup` name (0.2 pre-release, no external users); the
  subcommand is `rm`. One optional positional `target`.
- Selection: a target resolves ref-first (path, handle, slug — slug wins on a
  bare token), falling back to filter parse + `select` when it is not a ref. A
  filter may match many tasks; the existing confirmation prompt lists them.
- Gate: an explicit target bypasses the terminal-status eligibility gate —
  `rm <wip-task>` removes it. The terminal gate is now just the built-in default
  filter, reachable as `rm terminal`.
- Filter language gains a `terminal` predicate (bare or `terminal()`), matching
  tasks whose status is flagged terminal in config. Sibling to `ready`/`blocked`.
- Config: `[rm] default_filter` governs the no-argument run; built-in default
  `"terminal"`. Users reconfigure it.

== Decisions

- Explicit target bypasses the terminal gate (confirmed with operator). Removal
  stays recoverable: dirty-tree guard, confirmation prompt, SWHID tombstone,
  reviewed-then-committed by the user.
- Ref precedence over filter on an ambiguous bare token (a real slug wins).
- `typst-cleanup` crate keeps its name — consumer-blind, names the shared
  skeleton, not the user-facing verb.
