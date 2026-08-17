#import "@local/mindtape:0.2.0": *

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

MindTape shipped `assignee` and then never used it. The dogfood config carried one placeholder actor — `ada`, copied from the docs example — declared no `[tasks.assignee]` section at all, and not one task under `truth/tasks/` set the field. So the backlog could not answer the question that comes up every session: which open tasks need a human, and which can an agent take. `verify-the-generated-readme-header-renders-on-github.typ` is the shape of the problem — a task no gate and no agent can ever close, sitting in the same undifferentiated listing as work an agent should just pick up.

This adopts the field with two actors and two views: `mt ls @owner` is what waits on the maintainer, `mt ls @agent` is what an agent can take.

== Scope

- `[tasks.assignee]`: `allowed = ["max", "agent"]`, `required = "!terminal"` — the same shape as `priority`/`difficulty`/`estimate`, so an open task says who works it and a closed one never gets a name backfilled onto it
- `[[actors]]`: the placeholder `ada` becomes `max` (human, `maintainer`), `agent` joins as `llm`. Two names, because the only distinction this backlog needs is human-vs-model; which model takes an agent task is routing policy read off `difficulty`, which is also why no vendor or model name belongs in the registry
- `[views.owner]` and `[views.agent]`, both `!terminal` and `priority:desc`. The actor is `max` and only the LISTING is called `owner` — the field says who works a task, never who owns it
- the three open tasks backfilled; the 269 terminal ones are exempt by construction
- the four sweep templates hardcode `assignee: "agent"` — a swept finding is agent work by definition, so `contrib/ask-refactoring/*.sh` needs no new `--set` and cannot drift out of sync with the config
- `docs/src/skill.typ` documents the spliced slot generically: `--describe` lists an `assignee` no template declares, `$MINDTAPE_PRINCIPAL` fills it, and unset is an unknown actor rather than a default

== Why no default fill

No template declares `assignee`, so `mt add` splices in the env-backed choice slot, and a non-interactive add with no `$MINDTAPE_PRINCIPAL` fails naming the slot. A template-level `default: "agent"` would be more convenient and was rejected: humans and agents share one dev shell here, so any default is the mechanism by which the maintainer's own work gets silently filed as the agent's. A loud error costs one flag and cannot mis-attribute.

For the same reason the variable is not exported anywhere in the repo — not `.envrc`, not the flake devShell, both of which the maintainer's interactive shell and an agent's tool calls share. An agent passes `--set assignee=agent`; a harness that wants it silent sets the variable in its own agent-only config.

== Where the docs went

The repo-specific half — who `max` and `agent` are, why one `agent` actor is enough, why the views are `!terminal` — lives in `.mindtape/config.toml`'s own comments, next to the keys it explains. `docs/src/contributing.typ` sits at 100% of its linecop cap (260/260), so a process note there would have to buy its lines from existing prose; the generic mechanic went to `docs/src/skill.typ` instead, which had headroom. If this convention needs to be in CONTRIBUTING.md, that is a compaction pass on `docs/src/contributing.typ`, filed on its own.
