#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Eval the skill against the example projects",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 3,
  tags: ("tests",),
  estimate: (files: 5),
  status: wip(2026, 7, 26),
)

== Summary

The skill's evals ran against a bespoke five-task fixture that duplicated what examples/ already provides, and vendored a copy of the prelude that had to be kept in sync by hand. Point them at the example projects instead: learn-piano for realistic triage, and a new examples/minimal for the two traps learn-piano cannot express, since its legend declares blocked and it ships a template. Both are gated by check-examples, which is what earns the right to depend on them.

== Scope

- Promote the trap fixture to examples/minimal, dropping the vendored lib/ (the XDG-installed package resolves) so the prelude drift vector disappears with it.
- Retarget evals 0-3 onto learn-piano with assertions written against its real titles, scores and blocked split; keep 4-5 on minimal.
- Add evals/README.md recording the copy-not-symlink protocol and the model choice.
- Update check-examples, the mmz examples scope, outdatty and CONTRIBUTING for the new layout.
- Also correct the skill's hand-authoring rules: a bare `//` starts a Typst line comment and the forbid policy rejects it, while `https://` is safe because Typst auto-links it. This task tripped over exactly that while being filed.
