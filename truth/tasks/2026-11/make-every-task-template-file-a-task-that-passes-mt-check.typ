#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Write every goal verdict explicitly in tasks and templates",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 3,
  links: (
    (
      task: "lint-typst-prose-with-vale-through-typst2vast.typ",
      kind: "surfaced-by",
      note: none,
    ),
  ),
  tags: ("found", "tooling", "stateless"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[a template that files an invalid task breaks the add path at its first step],
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
      self-describing: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 11, 14),
)

== AS-IS

+ Open a task filed after `verdicts` landed, this one included.
+ 🔴 Every goal sits in one `verdicts(...)` call. A reader then chases a helper to find the one goal that helps.
+ 🔴 The helper let a template default a `helps[...]` reason the task never earned.

```
$ mt add -t feat "Probe the defaults" --print
# product-goals, product-neutral, process-goals, process-neutral
```


== TO-BE

- 🟢 Every task file lists `x-goals` one line per goal, `readable-tasks: helps[...]` or `cheap-adoption: neutral`.
- 🟢 Every template asks one rating per goal, defaulting `neutral`. The author then writes the `helps[...]` in their own words.
- 🟢 `mt add -t feat` with every slot at its default files an all-neutral task, and `mt check` refuses it until one goal says how it helps.
- 🟢 A goal added to `truth/goals.typ` fails `mt check` until every template carries a rating for it.

```
$ mt add -t feat "Show the due date first" --set readable-tasks='helps[who reads the list sees what is due first]'
```


== Changes

- `.mindtape/templates/*.typ`: one slot per goal, no `verdicts` call.
- `truth/tasks/**/*.typ`: rewrite every `verdicts` block to explicit per-goal dicts.
- `.mindtape/config.toml`: drop `[templates] check-defaults`, which existed for the all-neutral defaults.
- `examples/api-service/.mindtape/templates/milestone.typ`, `examples/api-service/goals.typ`: the explicit shape.
- `www/content/tie-work-to-goals.typ`, `skills/mindtape/references/hand-editing.md`: document it.
- `contrib/rewrite-x-goals.py`: remove the obsolete migration script.


== Out of scope

- Removing the `verdicts` helper stays out. It stays exported and a later release deprecates it.


== Dogfood

`mt add -t feat` with `--set readable-tasks='helps[...]'` files a task `mt check` accepts. The checker refuses the same add with no goal answer.

