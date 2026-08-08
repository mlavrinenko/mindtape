#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "give the size sweep a remedy per language",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 2,
  tags: ("tooling",),
  estimate: (files: 2),
  links: related(
    "file-refactoring-tasks-from-quality-signals.typ",
  )[the sweep this refines],
  status: done(2026, 8, 8),
)

== Summary

The size sweep splits findings two ways, test module or production, and hands
the production branch prose about carving a Rust file along a seam and not
leaving "an arbitrary bottom half moved into a `_more.rs`". linecop also caps
Markdown, so `CONTRIBUTING.md` and `skills/mindtape/SKILL.md` land on that
branch and would be given advice about Rust modules. Nothing is broken — at the
default `--baseline 90` no Markdown is reported today — but a machine-filed
task that talks about the wrong language is exactly the detail that teaches a
reader to stop believing the rest of it, which is the one failure this sweep
cannot afford.

== Scope

- Branch the remedy on the finding's language, which `linecop --format json`
  already reports as a field — do not infer it from the file extension a second
  time.
- The Markdown remedy is genuinely different: progressive disclosure into
  `references/`, splitting a section into its own page, or re-capping with a
  written reason. `.linecop.yaml` already argues this at length for the 300-line
  overrides — draw on that reasoning rather than restating it.
- The test-versus-production split stays, but it is a Rust distinction. Keep
  the two axes separate rather than growing one flat list of cases.
- Verify against a real Markdown finding by running the sweep at a baseline low
  enough to report one, the way the test branch was verified.
