#import "@local/mindtape:0.4.0": *
#import "/truth/lib.typ": *

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
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      config-over-code: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
      self-describing: helps[the harness measures the no-docs claim without the context of the orchestrator],
    ),
    process: (
      sound-codebase: helps[a clean harness makes the eval result reproducible],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 11, 20),
)

== AS-IS

+ Read `skills/mindtape/evals/README.md` and run one no-docs eval from the main checkout.
+ 🔴 The no-docs arm spawns a `general` subagent from this session, so it inherits the checkout, the installed skill, the personal config and the conversation context of the orchestrator.
+ Ask `opencode run` for its skill ids, with the real home and with a clean one.

```
$ opencode run --standalone "List the skill ids available to you"
agent-browser, ..., mindtape, ...
$ work=$(mktemp -d)
$ HOME=$work XDG_CONFIG_HOME=$work/.config opencode run --standalone "List the skill ids available to you"
opencode, report
```

+ 🔴 A clean home hides the installed skill, but `XDG_CONFIG_HOME` alone does not: `~/.claude`, `~/.agents`, the shared service and the repo's `AGENTS.md` still reach the executor.


== TO-BE

- 🟢 The harness is `opencode run --standalone` inside `bwrap`, with `HOME` and the four XDG dirs under the run directory, so no skill, config, session or repo checkout is visible.
- 🟢 The harness copies the fixture outside the repo and runs `mt install` in the sandbox home, then drops the fixture's `../../lib` override.
- 🟢 The harness pins the model with `--model` and records it beside the trace, so two runs compare.
- 🟢 `just skill-evals --arm no-docs` runs all six into `target/skill-evals/<run-id>/`, and a second sandboxed run grades each trace.

```
$ just skill-evals --arm no-docs
eval 0  pass   7/7
eval 1  pass   4/4
```


== Changes

- `truth/tasks/pick-an-isolated-harness-for-the-skill-evals.typ`: records the pick, the sandbox shape, the model pin and the tradeoffs. Only this file changes. The follow-up task builds `just skill-evals` and the grader.


== Alternatives

- Keep skill-creator's loop with a stricter prompt: the orchestrator still supplies repo access and context. The taint remains.
- `XDG_CONFIG_HOME` alone: hides the installed skill, but not `~/.claude`, `~/.agents`, the shared service or the repo at its absolute path.
- A container or VM: the same boundary as `bwrap` with more setup, and it still needs a model endpoint and a runner.
- An API model with no tools: the cleanest arm, but it cannot run `mt`.
- Scripted `mt` commands with no model: tests the CLI, not whether a consumer discovers it.
- `bwrap` shares the network, so the executor can reach the internet. The repo stays unreachable because nothing mounts it.


== Out of scope

- Building or running the harness stays out. A follow-up task implements `just skill-evals`, the grader and the results layout.
- Changing the eval prompts, the fixtures or the no-docs assertions.
- A macOS runner: `bwrap` is Linux-only and `aarch64-darwin` is a supported system, so that arm needs its own boundary.


== Dogfood

This repo runs the six evals in the harness before it relies on the `self-describing` baseline again, and records the run id beside the baseline table.
