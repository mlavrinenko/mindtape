#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Validate status transitions",
  priority: ice(
    impact: 4,
    confidence: 0.7,
    ease: 6,
  ),
  tags: (),
  status: proposed(2026, 7, 8),
)

== Summary

Add optional, config-declared allowed status transitions, enforced by `mt flip`
and `mt check`. Today any legend value is reachable from any other. dstask and
Taskwarrior model a lifecycle (pending → active → resolved); MindTape's legend is
a flat enum. This extends the "typed and validated" edge the comparison credits
to MindTape, and stays opt-in: declare transitions to get a state machine, omit
the section to keep today's any-to-any behaviour.

== Constraints

- Opt-in and backward-compatible — absent config means unconstrained, exactly as
  now. No existing backlog should start failing `mt check` on upgrade.
- Terminal states stay terminal unless an explicit reopen edge is declared.

== Scope

- Config: per-status `transitions` (allowed next values), or a top-level
  transition table; validate the declared graph is well-formed.
- `mt flip` rejects an illegal transition with a message listing the allowed next
  states; `mt check` flags any task whose current status is unreachable under the
  declared rules.
- Tests: legal and illegal flips, the unconstrained default, a declared reopen
  edge.
