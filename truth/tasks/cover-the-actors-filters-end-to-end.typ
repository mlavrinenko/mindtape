#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "cover the actors filters end to end",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 2,
  tags: ("cli", "tests"),
  estimate: (files: 1),
  status: done(2026, 8, 7)[landed 2026-08-07; just check green on merged HEAD],
  links: related("actors-roles-registry-manual-test.typ")[
    the session that surfaced this gap
  ]
    + related("actors-roles-registry.typ")[
      the implementation left uncovered at the CLI seam
    ],
)

== Summary

The `role` and `assignee` filters have library unit tests against a hand-built
`Config` only (`filter_actors_tests.rs`). Nothing exercises them through the
CLI, so a break anywhere between TOML parsing, project loading and `mt ls`
would ship green. The 2026-08-07 manual session was the first time they ran end
to end.

== Scope

- An e2e test under `crates/mindtape-cli/tests/` scaffolding a project with
  `[[roles]]` and `[[actors]]`, tasks assigned across them, and one assigned to
  an undeclared name.
- Assert `mt ls 'role == "reviewer"'` lists exactly the tasks whose assignee
  holds that role, `mt ls 'assignee == "gpt"'` matches by actor name, and an
  undeclared assignee resolves no role rather than falling through to a match.
- Assert `mt check` surfaces a registry violation (undeclared role, non-actor
  in `allowed`) as a config load error.
