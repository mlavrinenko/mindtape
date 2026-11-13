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
  status: done(2026, 11, 14),
)

== AS-IS

+ Filing the five Vale follow-ups from the `feat` template
+ 🔴 #raw(".mindtape/templates/feat.typ"): the default product-neutral list omits readable-tasks, so mt add -t feat files a task that mt check refuses
+ 🔴 `mt check` stays green in the same tree, because it files each template with its examples and never with its defaults

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

- 🟢 `mt add -t feat` with every slot at its default files a task `mt check` accepts.
- 🟢 `readable-tasks` gets a `helps[...]` verdict by default, and a task that helps it overrides that without editing the neutral list.
- 🟢 `[templates] check-defaults` makes `mt check` file each template with its defaults too, so a default that falls behind its example fails. This repo turns it on; it is off by default for a template that leaves a goal neutral on purpose.
- 🟢 Adding a goal to `truth/goals.typ` fails `mt check` until every template's defaults rate it.

```
$ mt add -t feat "Probe the template defaults" --print
# x-goals has a readable-tasks verdict, so mt check accepts it
```


== Changes

- `.mindtape/templates/*.typ`: `readable-tasks: helps[...]` as the default `product-goals` verdict; `finding.typ` stops putting `readable-tasks` in its neutral list.
- `.mindtape/config.toml`: `[templates] check-defaults = true`.
- `crates/mindtape/src/**/*.rs`: `fill_defaults` beside `fill_examples`, and `[templates] check-defaults` on `TemplatesConfig`, with the red e2e test in `crates/mindtape-cli/tests/it/check_templates_e2e.rs`.
- `crates/mindtape/src/config/defaults.toml`, `schema/config.schema.json`: the new key and its blessed schema.
- `crates/mindtape-cli/src/cmd/check/templates.rs`: file each template with its examples, and with its defaults when the project asks; one breach each.
- `www/content/templates.typ`, `www/content/rules.typ`, `www/content/cli/check.typ`: say what `mt check` files now.


== Out of scope

- The goal list in `truth/goals.typ` and the `verdicts` helper. The template defaults are the bug.


== Dogfood

A fresh `mt add -t feat` passes `mt check` with no edit to its `x-goals`, and adding a goal to `truth/goals.typ` fails `mt check` until the templates rate it.

