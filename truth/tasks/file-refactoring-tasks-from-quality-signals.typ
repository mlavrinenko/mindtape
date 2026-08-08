#import "@local/mindtape:0.2.0": *

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
  status: proposed(2026, 8, 8),
)

== Summary

Mechanical refactoring work recurs and is currently noticed by a human at the moment it blocks something else: six file-size splits landed in the two months since linecop was adopted, each filed by hand, and the one filed preemptively said so in its own body. The advisory signals that predict this work are already machine-readable — linecop has `--format json`, cargo-crap has `--format json` with a `--baseline` delta, cargo-mutants writes `mutants.out/outcomes.json`. Nothing reads them into the backlog. This task builds the sweep and lands the first signal; the other signals are siblings.

== Scope

- `contrib/ask-refactoring.sh`, wrapped by `just ask-refactoring`. Dry by default: it prints the tasks it would file, via `mt add --print`. `--write` files them.
- Only advisory signals qualify. A signal that already blocks a commit — jscpd, outdatty, machete, clippy — must never produce a task, because the wall is already in front of you and the task would be a note about it.
- One task per finding, never one task per sweep: each is dispatched to its own agent and must carry its own scope.
- Re-run guard: the sweep refuses to file while the view `ask-refactoring` is non-empty, so a sprint is emptied before the next one is filed. Add `[views.ask-refactoring]` filtering on the provenance tag and non-terminal status, and add the tags to `[tasks.tags] allowed` — a shared provenance tag plus one per signal.
- Idempotency comes from the signal, not from a ledger of what was filed. A closed task whose condition still holds should be re-filed; the work did not happen. The deliberate suppression is a `.linecop.yaml` override with a written reason, which is the mechanism that file already uses for SKILL.md and AGENTS.md.
- First signal: linecop. Split test files from production files in the output — both need a real seam decision now that the ejected modules are themselves at the cap, but they are different work and should not share a task.
- The remedy is the refactorer’s call and the template must say so: split thoughtfully, or raise the limit with an override that states a new sane number and why. A split for its own sake is not the goal.
- Score honestly. The config runs strict ICE, so the sweep must supply impact, confidence and ease. Confidence is high because the measurement is exact rather than guessed; impact scales with how little headroom is left; ease follows from the signal.
- `jq` moves from a test and docs convenience into a real dev-shell dependency.
- Three templates under `.mindtape/templates/`, one per signal, plus an outdatty group holding their shared shape in step. A Typst `#import` would leak into every written task and bind it to a path outside the tasks root, so the sharing is a drift gate, not an import.
