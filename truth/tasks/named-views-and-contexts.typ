#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Named views and contexts",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 6,
  ),
  tags: (),
  status: proposed(2026, 7, 8),
)

== Summary

Let the project name and persist queries. Today every `mt ls` beyond the default
is an ad-hoc filter string retyped each time. The comparison shows dstask has
contexts (a filter and tags applied automatically to queries and new tasks) and
Taskwarrior has saved reports and contexts; MindTape has neither. This is on
brand because it is config-driven — the filter language already exists, so a view
is just a named preset declared in `.mindtape/config.toml`, not new machinery.

== Scope

- `[views]` table: name maps to a filter expression, plus optional sort and
  column set. `mt ls --view <name>`; `mt ls --list-views`.
- A view composes with an explicit filter argument (AND), so presets narrow
  rather than replace.
- Optional `[context]`: a named filter (and default tags for new tasks) applied
  until switched — `mt context <name>` / `mt context none`. Decide where the
  active context lives so the daemonless model holds: a tracked marker file, or
  XDG state. Views (stateless) are the core; context (stateful) is the follow-on.
- Tests: view resolution, precedence with an explicit filter, unknown-view error.
