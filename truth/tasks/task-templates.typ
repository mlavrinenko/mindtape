#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Move the templates to AS-IS, statefulness and a decision kind",
  kind: "dev",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 2,
  tags: ("mt",),
  estimate: (src-files: 14),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: helps[a slot prompt shows the new shape at filing],
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 11, 1),
  links: depends-on("task-section-rules.typ"),
)


== Problem

+ Run `mt add --describe -t dev` and read the sections the template asks for.
+ 🔴 Every template still writes `Problem`, and the `dev` template still writes `Rollback`, so the new rules have no template that satisfies them.
+ 🔴 A `stateful` feat or fix has no template that carries the Rollback its tag requires.

== TO-BE

- 🟢 The templates write `AS-IS` and default `stateless`, and `dev-stateful`, `feat-stateful` and `fix-stateful` carry `Rollback`.
- 🟢 `feat`, `fix` and `dev` prompt for the statefulness tag and a fenced example in `TO-BE`.
- 🟢 A `decision` template carries `Alternatives` and names the task file in `Changes`; `finding` and the generic `task` default to `stateless` too.

```typ
#show: task.with(title: "…", kind: "dev", tags: ("stateless",), status: wip(2026, 11, 1))
```

== Changes

- `.mindtape/templates/*.typ`: the headings, the statefulness tags and the example fences.
- `.mindtape/templates/dev-stateful.typ`: the stateful dev template with Rollback.
- `.mindtape/templates/feat-stateful.typ` and `.mindtape/templates/fix-stateful.typ`: the same for feat and fix.
- `.mindtape/templates/decision.typ`: the decision template.

== Rollback

- Revert the templates; the old section prompts return.

== Out of scope

- The pack rules in `task-section-rules`, the docs in `task-shape-docs` and the corpus in `as-is-sweep`.

== Dogfood

`mt add` in this repo uses these templates for every new task.
