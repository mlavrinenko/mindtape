#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "sweep surviving mutants into tasks",
  priority: ice(
    impact: 7,
    confidence: 0.7,
    ease: 4,
  ),
  difficulty: 6,
  tags: ("tooling", "tests"),
  estimate: (files: 5),
  links: depends-on("file-refactoring-tasks-from-quality-signals.typ"),
  status: proposed(2026, 8, 8),
)

== Summary

A surviving mutant is a line the suite covers but does not pin — a bad test, stated precisely. That is the signal mutation testing exists to give and the one this project has no standing mechanism to act on. CONTRIBUTING deliberately keeps mutants out of \`just check\` because an advisory gate with unreliable verdicts trains people to ignore it; a sprint-shaped backlog is a different consumer, since each finding gets read once and either becomes work or becomes a named exclusion.

== Scope

- Read \`mutants.out/outcomes.json\` (written unconditionally by any sweep) and file one task per surviving mutant, naming the file, the line and the mutation that lived.
- Scoping is the open decision. A full workspace sweep is hours; \`--in-diff\` against a recorded last-sweep commit bounds it to what changed and matches the weekly cadence. Record that marker somewhere durable and say in the task body what range was swept, so a finding is never read as covering more than it does.
- A finding that is a genuine equivalent mutant becomes a named \`exclude\_re\` in \`.cargo/mutants.toml\` with its reason, as the two existing entries already do. That is the disposal path and the template must name it, so cancelling a task is a real outcome rather than a silent drop.
- Memoize the expensive producers with mmz so a re-swept, unchanged tree costs nothing.
