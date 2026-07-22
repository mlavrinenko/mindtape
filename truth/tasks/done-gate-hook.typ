#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "done gate hook",
  priority: ice(
    impact: 7,
    confidence: 0.65,
    ease: 4,
  ),
  tags: (),
  status: proposed(2026, 7, 22),
)

== Summary

Let a project attach a machine-checkable command to a status transition, so a terminal stamp can be refused unless something real passes. This is the constraint with the best evidence behind it. Across 20574 agent sessions, inaccurate self-reporting accounted for 22.58 percent of misalignment episodes and 91.49 percent of resolved misalignments required explicit human pushback — agents essentially do not self-correct. The mechanism practitioners report as actually catching false completions is an external predicate, not a written justification: a model asked to justify a claim predicts a plausible ending rather than checking reality. A note field cannot police the thing that writes the note.

== Scope

- New `[[tasks.gates]]` table: `when` (filter over the task), `into` (target status, optional), `run` (command), `message`.
- `mt flip` evaluates matching gates before writing. Non-zero exit refuses the transition and prints the command output. `--force` overrides and is recorded in the status note.
- Only `mt flip` runs gates. `mt check`, `mt ls` and every other read command never execute configured commands, so inspecting a repository stays side-effect free.
- Trust boundary is the same as a Justfile or a git hook: the command comes from the in-repo config. Document it plainly in the gate reference and in `mt init --annotate` output; do not pretend it is sandboxed.
- Gate commands receive the task path and stem in the environment so a script can inspect the artifact.
- Tests: passing gate allows the flip, failing gate refuses and surfaces output, --force overrides, gates scoped by `when` and `into` fire only on matching transitions, read commands never execute anything.
