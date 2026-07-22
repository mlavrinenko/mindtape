#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "mmz freshness in gates",
  priority: ice(
    impact: 5,
    confidence: 0.6,
    ease: 6,
  ),
  difficulty: 3,
  tags: ("tooling", "gating"),
  status: proposed(2026, 7, 22),
  links: depends-on("flip-hooks.typ"),
)

== Summary

Wire this repository's own close gate to `mmz --is-fresh` so that flipping a task to a terminal status asserts the checks already ran against the current worktree, instead of running them again.

A gate that runs `just check` inside `mt flip` is bad ergonomics: the command is slow, it runs at the least convenient moment, and an agent learns to reach for `--force`. A gate that merely asserts the command already ran against this state is cheap, and it is the machine-checkable predicate the review is actually asking for — proof of a passing build, not a claim of one.

This is harness configuration, not a MindTape feature. `[[on.flip]]` stays tool-agnostic with a plain `run` command; `mmz` is one thing that command can be, and a project without `mmz` points `run` at anything else. Nothing here reaches consumers.

== Scope

- Add an `[[on.flip]]` gate to `.mindtape/config.toml` firing on the transition into a terminal status: `run = "mmz --is-fresh -- just check"`, `gate = true`.
- The failure message names the fix — run `just check`, then flip — rather than reporting a stale cache.
- `--is-fresh` is new and lightly exercised. Verify by hand before trusting it in a gate: confirm it exits non-zero after a source edit, exits zero after a real run, and never itself runs the wrapped command. If it misbehaves, the fix belongs in `mmz`'s own backlog, not worked around here.
- Decide what counts as the meaningful worktree slice for the rule. Task artifacts under `truth/tasks` change on every flip, so a rule keyed on the whole tree would go stale the moment a status is stamped and the gate would never pass.
- Document the pattern in the `gated` preset's annotated comments as the recommended way to make a close gate cheap, naming `mmz` as one option rather than a requirement.
- Tests: this one is manual and says so. The behaviour under test is another tool's cache, and a unit test asserting `mmz` semantics would test the mock, not the tool.
