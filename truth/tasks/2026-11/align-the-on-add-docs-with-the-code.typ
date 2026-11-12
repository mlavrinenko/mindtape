#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Align the on.add docs with the code",
  kind: "dev",
  priority: ice(
    impact: 3,
    confidence: 0.95,
    ease: 8,
  ),
  difficulty: 1,
  links: (
    (
      task: "let-rules-and-templates-choose-where-new-tasks-land.typ",
      kind: "surfaced-by",
      note: none,
    ),
  ),
  tags: ("found", "mt", "docs", "stateless"),
  x-goals: (
    product: verdicts(
      (:),
      ..(
        "readable-tasks",
        "implementable-tasks",
        "cheap-adoption",
        "typst-over-markdown",
        "plain-files",
        "self-describing",
      ),
    ),
    process: verdicts(
      (:),
      ..("sound-codebase", "docs-match-code"),
    ),
  ),
  status: done(2026, 11, 12),
  assignee: "agent",
)

== AS-IS

+ Reviewing the on-add branch after `just check`.
+ 🔴 #raw("www/content/hooks.typ"): The docs still said `old()` refused only id/path/dir, described a derive's inputs too narrowly, and repeated an explicit `--set` win that `+=` does not have.

The page said `It refuses id, path and dir`, while the code also refuses `file`, the graph fields, derived tags and `derived.<name>`; the same page said `An explicit --set ... wins over a hook set`, which is false for `--set field+=`.


== TO-BE

- 🟢 The docs name the draft-unsafe fields `old()` refuses.
- 🟢 They say an explicit `=` wins and an explicit `+=` does not.
- 🟢 They state `--dir`'s relative and absolute rules, the `--describe` headings, and the current eval expectations.


== Changes

- `www/content/{hooks,filters,templates}.typ`, `www/content/cli/{add,import,set,explain}.typ`, `www/content/agents.typ`, `www/utils/cli-notes.typ`: align the prose.
- `skills/mindtape/references/{filters,add-slots}.md`, `skills/mindtape/evals/README.md`: align the skill and eval notes.
- `CHANGELOG.md`: correct the add-hook entry.


== Out of scope

- The code changes filed in the other tasks.


== Dogfood

The docs describe the branch as it behaves.

