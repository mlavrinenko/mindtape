#import "@local/mindtape:0.3.0": *

#show: task.with(
  title: "file refactoring tasks from quality signals",
  priority: ice(
    impact: 8,
    confidence: 0.8,
    ease: 4,
  ),
  difficulty: 6,
  tags: ("tooling", "gating"),
  estimate: (files: 12),
  links: depends-on(
    "add-dedup-modes-to-mt-add.typ",
    "give-mt-add-machine-readable-output.typ",
  ),
  status: done(
    2026,
    8,
    8,
  )[sweep + linecop signal landed. Dry run reproduces 7 findings, guard observed refusing on a non-empty view, test/production split verified at `--baseline 85`],
)

== Summary

Mechanical refactoring work recurs, and a human notices it only when it blocks
something else. Six file-size splits landed in the two months since linecop
adoption, each filed by hand, and the one filed preemptively said so in its own
body. The advisory signals that predict this work are already machine-readable:
linecop has `--format json`, cargo-crap has `--format json` with a `--baseline`
delta, and cargo-mutants writes `mutants.out/outcomes.json`. The backlog reads
none of them. This task builds the sweep and lands the first signal. The other
signals are siblings.

== Scope

- `contrib/ask-refactoring.sh`, wrapped by `just ask-refactoring`. Dry by
  default: it prints the tasks it would file, via `mt add --print`. `--write`
  files them.
- Only advisory signals qualify. A signal that already blocks a commit (jscpd,
  outdatty, machete, clippy) must never produce a task. The wall is already in
  front of you, and the task would be a note about it.
- One task per finding, never one task per sweep. Each finding goes to its own
  agent and must carry its own scope.
- Re-run guard: the sweep refuses to file while the view `ask-refactoring` is
  non-empty. A sprint then empties before the next one starts. Add
  `[views.ask-refactoring]` filtering on the provenance tag and non-terminal
  status, and add the tags to `[tasks.tags] allowed`: a shared provenance tag
  plus one per signal.
- Idempotency comes from the signal, not from a ledger of earlier filings. A
  closed task whose condition still holds should be re-filed, because the work
  did not happen. The deliberate suppression is a `.linecop.yaml` override with a
  written reason, the mechanism that file already uses for SKILL.md and
  AGENTS.md.
- First signal: linecop. Split test files from production files in the output.
  Both need a real seam decision, because the ejected modules themselves sit at
  the cap. They are different work and should not share a task.
- The remedy is the refactorer's call, and the template must say so. Split
  thoughtfully, or raise the limit with an override that states a new sane number
  and why. A split for its own sake is not the goal.
- Score the findings. The config runs strict ICE, so the sweep must supply
  impact, confidence and ease. Confidence is high because the measurement is
  exact rather than guessed. Impact scales with how little headroom remains, and
  ease follows from the signal.
- `jq` moves from a test and docs convenience into a real dev-shell dependency.
- Three templates under `.mindtape/templates/`, one per signal, plus an outdatty
  group holding their shared shape in step. A Typst `#import` would leak into
  every written task and bind it to a path outside the tasks root, so the sharing
  is a drift gate, not an import.
