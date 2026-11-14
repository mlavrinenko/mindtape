#import "@local/mindtape:0.3.0": *
#import "/truth/lib.typ": *

#show: task.with(
  title: "Rewrite the README around what Typst tasks express",
  priority: ice(
    impact: 9,
    confidence: 0.7,
    ease: 5,
  ),
  difficulty: 5,
  tags: ("docs", "www"),
  estimate: (files: 3),
  kind: "docs",
  assignee: "agent",
  status: done(2026, 9, 16),
  links: depends-on("move-goal-verdicts-into-x-goals.typ"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[an engineer sees the reason to switch before the install steps],
      typst-over-markdown: helps[the first screen shows what only a Typst task can say],
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
)

== AS-IS

+ Open `README.md` as a software engineer who already keeps tasks in Markdown files with YAML frontmatter.
+ 🔴 The first screen calls mindtape a daemon-free file-based task tracker and shows a task, but nothing in it needs Typst: frontmatter plus a script does the same.
+ 🔴 The page never shows what Typst adds. `wip(2026, 9, 14)[note]` is one typed value, the tool checks `depends-on("a.typ")` edges on every command, and a Typst function the repo keeps beside its tasks checks an `x-goals` field.


== TO-BE

- 🟢 The README's first screen shows one task twice, as Markdown with YAML and as Typst. It states what only the Typst one does: typed values, links checked on every command, and a field the repo's own Typst function checks.
- 🟢 `www/content/index.typ` opens with the same comparison.

== Source of truth

- A real task in `truth/tasks/`: the side-by-side shows that file as it is, not an invented one.
- `truth/lib.typ`: the check function the README quotes.


== Changes

- `docs/src/readme.typ`: the side-by-side opening, with the task taken from `truth/tasks/`. `just docs md` regenerates `README.md`.
- `www/content/index.typ`: the same opening.
- `outdatty.lock`: re-record the group `docs/src/readme.typ` belongs to.


== Out of scope

- `www/content/comparison.typ`: it compares trackers feature by feature, a different question.
- A new docs page: `add-a-docs-page-on-tasks-that-validate-themselves.typ`.


== Dogfood

Built in: the side-by-side task is a real task from this repo's `truth/tasks/`.

