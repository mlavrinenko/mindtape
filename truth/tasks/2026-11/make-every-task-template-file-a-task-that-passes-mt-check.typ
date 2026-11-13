#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Make every task template file a task that passes mt check",
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
    product: verdicts(
      (
        enforced-practice: helps[a template that files an invalid task breaks the add path at its first step],
      ),
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
  assignee: "agent",
  status: proposed(2026, 11, 14),
)

== AS-IS

+ Filing the five Vale follow-ups from the `feat` template
+ 🔴 #raw(".mindtape/templates/feat.typ"): the default product-neutral list omits readable-tasks, so mt add -t feat files a task that mt check refuses

```
$ mt check
error: truth/tasks/2026-11/probe-the-feat-template-defaults.typ:3: rules.give-every-goal-verdict: product goal `readable-tasks` is not addressed; say helps[...], hurts[...] or neutral
error: truth/tasks/2026-11/probe-the-feat-template-defaults.typ:3: rules.give-every-goal-verdict: the task helps no goal; if nothing justifies it, cancel it
```

`.mindtape/templates/feat.typ` starts its `product-neutral` default at
`implementable-tasks`, so `readable-tasks` never gets a verdict. The same list
is in `fix.typ`, `docs.typ`, `dev.typ` and `decision.typ`; only `finding.typ`
includes it.


== TO-BE

- 🟢 `mt add -t feat` with every slot at its default files a task that `mt check` accepts.
- 🟢 `readable-tasks` gets a verdict by default, and a task that helps it overrides that verdict without editing the neutral list.
- 🟢 The five templates that share the list (`feat`, `fix`, `docs`, `dev`, `decision`) agree with `finding.typ`.

```
$ mt add -t feat "Probe the template defaults" --print
# x-goals has a readable-tasks verdict, so mt check accepts it
```


== Changes

- `.mindtape/templates/feat.typ`: give `readable-tasks` a default verdict.
- `.mindtape/templates/fix.typ`: the same.
- `.mindtape/templates/docs.typ`: the same.
- `.mindtape/templates/dev.typ`: the same.
- `.mindtape/templates/decision.typ`: the same.


== Out of scope

- The goal list in `truth/goals.typ` and the `verdicts` helper. The template defaults are the bug.


== Dogfood

A fresh `mt add -t feat` passes `mt check` with no edit to its `x-goals`.

