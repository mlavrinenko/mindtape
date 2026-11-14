#import "@local/mindtape:0.3.0": *

#show: task.with(
  title: "adopt the assignee field in the dogfood backlog",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 2,
  assignee: "agent",
  tags: ("mt", "docs"),
  estimate: (files: 10),
  links: related("assignee-field-core.typ")[the field this finally uses]
    + related("actors-roles-registry.typ")[the registry it points into]
    + related("render-the-assignee-as-an-ls-column.typ")[
      the gap adopting it exposed
    ],
  status: done(
    2026,
    8,
    18,
  )[adopted: two actors, required !terminal, \@owner/\@agent views],
)

== Summary

The tool gained `assignee` and then never used it. The dogfood config carried one
placeholder actor, `ada`, copied from the docs example. It had no
`[tasks.assignee]` section at all, and no task under `truth/tasks/` set the
field. So the backlog could not answer the question that comes up every session:
which open tasks need a human, and which can an agent take.
`verify-the-generated-readme-header-renders-on-github.typ` shows the problem: a
task no gate and no agent can ever close, sitting in the same
undifferentiated listing as work an agent should just pick up.

This adopts the field with two actors and two views: `mt ls @owner` is what waits
on the maintainer, and `mt ls @agent` is what an agent can take.

== Scope

- `[tasks.assignee]`: `allowed = ["max", "agent"]`, `required = "!terminal"`, the
  same shape as `priority`/`difficulty`/`estimate`. An open task says who works
  it, and a closed one never gets a name backfilled onto it.
- `[[actors]]`: the placeholder `ada` becomes `max` (human, `maintainer`), and
  `agent` joins as `llm`. Two names, because the only distinction this backlog
  needs is human vs model. Which model takes an agent task is routing policy read
  off `difficulty`, which is also why the registry carries no vendor or model
  name.
- `[views.owner]` and `[views.agent]`, both `!terminal` and `priority:desc`. The
  actor is `max`, and only the listing goes by `owner`: the field says who works
  a task, never who owns it.
- Backfill the three open tasks. The 269 terminal ones are exempt by
  construction.
- The four sweep templates hardcode `assignee: "agent"`. A swept finding is agent
  work by definition, so `contrib/ask-refactoring/*.sh` takes no new `--set` and
  cannot drift out of sync with the config.
- `docs/src/skill.typ` documents the spliced slot generically: `--describe` lists
  an `assignee` no template declares, `$MINDTAPE_PRINCIPAL` fills it, and unset
  is an unknown actor rather than a default.

== Why no default fill

No template declares `assignee`, so `mt add` splices in the env-backed choice
slot, and a non-interactive add with no `$MINDTAPE_PRINCIPAL` fails naming the
slot. A template-level `default: "agent"` would be more convenient and was
rejected. Humans and agents share one dev shell here, so any default is the
mechanism by which the maintainer's own work gets silently filed as the agent's.
A loud error costs one flag and cannot mis-attribute.

For the same reason the variable is not exported anywhere in the repo. That
includes `.envrc` and the flake `devShell`, both of which the maintainer's
interactive shell and an agent's tool calls share. An agent passes `--set assignee=agent`. A
The harness that needs silence sets the variable in its own agent-only config.

== Docs placement

The repo-specific half lives in `.mindtape/config.toml`'s own comments, next to
the keys it explains. Those comments say who `max` and `agent` are, why one
`agent` actor is enough, and why the views are `!terminal`. `docs/src/contributing.typ` sits at 100% of its
linecop cap (260/260), so a process note there would have to buy its lines from
existing prose. The generic mechanic went to `docs/src/skill.typ` instead, which
had headroom. If this convention needs to be in CONTRIBUTING.md, that is a
compaction pass on `docs/src/contributing.typ`, filed on its own.
