#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "format the project's own templates",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 2,
  tags: ("tooling",),
  estimate: (files: 5),
  links: related(
    "format-all-source-files-in-just-fmt-not-only-rust.typ",
  )[the recipe this completes],
  status: done(
    2026,
    8,
    8,
  )[one list feeds both fmt recipes; three templates reformatted],
)

== Summary

`just fmt` and `just fmt-check` run `typstyle` over `lib`, `truth/tasks`,
`examples/learn-piano/tasks`, `examples/learn-piano/.mindtape/templates`,
`www/content`, `www/templates` and `www/utils` — every Typst directory in the
repo except one. `.mindtape/templates`, the project's own templates, is not in
either list.

So the example project's templates are formatted and the real ones are not.
`typstyle --check .mindtape/templates` reports all four files today. Nothing is
broken, but an edit there is silently outside the formatter, and the omission
reads as deliberate when it is almost certainly a copy of the learn-piano path
that never got its sibling.

== Scope

- Add `.mindtape/templates` to both `fmt` and `fmt-check`. They are two separate
  lists that must stay in step — that is the actual failure mode here, and it
  will not be the last time, so consider whether one list feeding both is worth
  it while in there.
- Reformatting will touch all four templates including `sweep-size.typ`, which is
  the source of the `sweep-templates` outdatty group, so its three dependents
  need reconciling and the lock re-recording in the same commit.
- The `Justfile` is a source in the `dev-docs` group, so `CONTRIBUTING.md` and
  `AGENTS.md` come with it. CONTRIBUTING's Quality Gates table spells out which
  Typst sets `fmt-check` covers and would be wrong otherwise.
- Check whether `fmt-fixture-check` needs the same addition; it asserts both
  formatters reject drift and may enumerate the same paths a third time.
