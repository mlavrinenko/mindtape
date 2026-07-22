#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "flip hooks",
  priority: ice(
    impact: 9,
    confidence: 0.7,
    ease: 3,
  ),
  difficulty: 7,
  tags: ("cli", "gating"),
  status: proposed(2026, 7, 22),
)

== Summary

One user-extensible mechanism for "when a task changes status, do something". Replaces two separately-filed features that were the same idea seen twice: printing the open task set on the way into work, and running a machine-checkable command before a task may close.

Both are actions on a flip. Shipping them as separate tables would give MindTape two mechanisms for one concept, and every later behaviour would have to pick a side. One table, several action kinds, and the project decides what fires where — nothing is hardcoded.

Why it matters that a close can be gated: agents report success they have not achieved and do not catch it themselves. In one study of real agent sessions, inaccurate self-reporting was roughly a fifth of all misalignment episodes and the overwhelming majority of corrections came from a human pushing back rather than the agent noticing. A written justification cannot police the thing that writes it — a model asked to justify a claim predicts a plausible ending rather than checking reality. An external predicate can.

== Scope

- New `[[on.flip]]` table: `when` (a filter expression over the task, plus an `into("status")` predicate for the target status) and `actions` (a list).
- Action kinds:
  - `{ show = "text" }` — print a message.
  - `{ list = "filter or @view", title = "..." }` — print matching tasks. This is how an agent starting work sees what else is in flight, with no materialised index and nothing written to disk.
  - `{ run = "command", gate = false, message = "..." }` — execute. With `gate = true` a non-zero exit refuses the flip and prints the command output; `--force` overrides and records the override in the status note. Without `gate` the output is informational.
- Only `mt flip` runs actions. `mt check`, `mt ls` and every other read command never execute configured commands — a read that shells out is a surprise, and `mt check` is what runs inside editors and commit hooks.
- Actions run in declaration order; the first failing gate stops the flip and nothing is written.
- Gate commands receive the task path and stem in the environment so a script can inspect the artifact.
- Trust boundary is the same as a Justfile or a git hook: the command comes from the in-repo config. Say so plainly in the reference and in `mt init --annotate` output. Do not imply it is sandboxed.
- Suppress `show` and `list` output under `--quiet` and when stdout is not a terminal, so scripted flips stay clean. Gates still run.
- The `gated` preset ships opinionated defaults: the open set on the way into the in-flight state, and a gate on the way into a terminal one.
- Nothing is materialised. The historical open set is already recoverable — tasks are files in git, so `mt ls` at the flipping commit reconstructs it exactly.
- Tests: a passing gate allows the flip and a failing one refuses it and surfaces output; `--force` overrides; `when` and `into(...)` scope actions to matching transitions only; `show` and `list` render and are silent when piped; read commands execute nothing; actions run in order and a failed gate skips the rest.
