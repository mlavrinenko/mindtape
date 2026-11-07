#import "@local/mindtape:0.4.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Pick an isolated harness for the skill evals",
  kind: "decision",
  priority: ice(
    impact: 6,
    confidence: 0.6,
    ease: 3,
  ),
  difficulty: 3,
  tags: ("tests", "tooling", "stateless"),
  x-goals: (
    product: verdicts(
      (
        self-describing: helps[the eval decides how to measure the no-docs claim without the orchestrator's context],
      ),
      ..(
        "readable-tasks",
        "implementable-tasks",
        "enforced-practice",
        "cheap-adoption",
        "typst-over-markdown",
        "plain-files",
      ),
    ),
    process: verdicts(
      (
        sound-codebase: helps[a clean harness makes the eval result reproducible],
      ),
      ..("docs-match-code",),
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 11, 7),
)

== AS-IS

+ Read `skills/mindtape/evals/README.md` and run one no-docs eval from the main checkout.
+ 🔴 The no-docs arm spawns a `general` subagent from this session, so it inherits the orchestrator's checkout, installed skill, personal config and conversation context. The result is a reproduction by a tool that already knows the repo, not a blind consumer.

```
$ rg -n 'executor subagent|skill-creator' skills/mindtape/evals/README.md
3:Six evals in `evals.json`, run through skill-creator's loop: spawn an executor
4:subagent per eval twice, once with the skill and once with no external
```

+ The same agent can read `skills/`, `docs/` and the main repo unless the prompt forbids it, and nothing verifies the prohibition.


== TO-BE

- 🟢 The repo runs each eval in a harness with a clean home, a scratch fixture and no skill, docs or repo checkout visible to the executor.
- 🟢 The harness pins the model and records its name, so two runs are comparable.
- 🟢 The grader sees the prompt, the fixture and the trace, not this repo's context.
- 🟢 The chosen design names how to run all six evals and where their results live.

```
$ just skill-evals --arm no-docs
eval 0  pass   7/7
eval 1  pass   4/4
```


== Changes

- `truth/tasks/pick-an-isolated-harness-for-the-skill-evals.typ`: records the harness choice, the run recipe and the tradeoffs; no code changes.


== Alternatives

- Keep skill-creator's loop with a stricter prompt: the orchestrator still supplies repo access and context, so the taint remains.
- Run in a container or VM with a clean home and only the fixture: isolates files and config, but needs a model endpoint and a runner.
- Send the prompt and fixture to an API model with no tools: the cleanest arm, but it cannot run `mt` unless the harness gives it a shell.
- Replay scripted `mt` commands with no model: tests the CLI, but not whether a consumer discovers the commands.


== Out of scope

- Building or running the chosen harness; a follow-up task implements the pick.
- Changing the eval prompts, the fixtures or the no-docs assertions.


== Dogfood

This repo runs the six evals in the chosen harness before it trusts the `self-describing` baseline again.

