#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Build the isolated skill-eval harness",
  kind: "feat",
  priority: ice(
    impact: 7,
    confidence: 0.8,
    ease: 4,
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
      self-describing: helps[the harness is what makes the no-docs arm blind],
    ),
    process: (
      sound-codebase: helps[a reproducible harness makes the eval result trustworthy],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 11, 20),
  links: related(
    "pick-an-isolated-harness-for-the-skill-evals.typ",
  )[implements the pick recorded there]
    + depends-on("ship-the-skill-eval-env-schema.typ"),
)

== AS-IS

+ Read the decision in `pick-an-isolated-harness-for-the-skill-evals.typ`.
+ 🔴 `just skill-evals` does not exist. The six evals still run through skill-creator's loop, which spawns a subagent from the session of the orchestrator.

```
$ just skill-evals --arm no-docs
error: Justfile does not contain recipe `skill-evals`
```

+ 🔴 Every run inherits the fixture, the home and the model name from the orchestrator.

== TO-BE

- 🟢 `just skill-evals --arm no-docs|with-skill|both` runs the six evals, each in its own `bwrap` sandbox.
- 🟢 Each run copies the fixture outside the repo, runs `mt install` in the sandbox home and puts one `mt` on `PATH`.
- 🟢 The recipe reads `.env` and refuses a placeholder. `SKILL_EVALS_MODEL_ID` specifies the model and `SKILL_EVALS_API_KEY` holds its key.
- 🟢 The executor is `opencode run --standalone --format json`, and `run.json` records the model, the `opencode` version, the mt version, `MINDTAPE_NOW` and the commit.
- 🟢 A second sandboxed run grades the trace against the eval's assertions into `verdict.json`.
- 🟢 Results land in `target/skill-evals/<run-id>/eval-<n>/<arm>/`.

```
$ just skill-evals --arm no-docs
eval 0  pass   7/7
eval 1  pass   4/4
run: target/skill-evals/2026-11-20T17-10Z
```

== Changes

- `.just/scripts/skill-evals.sh`: the harness: sandbox, fixture copy, `mt install`, executor run, grader run, results.
- `Justfile`: the `skill-evals` recipe, with `--arm` and `--eval`.
- `nix/dev/shell.nix`: add `bubblewrap` to the dev shell.
- `skills/mindtape/evals/README.md`: replace "Running one" with the recipe, the `.env` variables and the results layout.

== Alternatives

- A Rust subcommand `mt skill-evals`: puts a model runner in the domain library, which has no HTTP client for a model API.
- A shell script without `bwrap`: the decision measured the leak.
- Grading by the orchestrator: the grader would see this repo.

== Out of scope

- Changing the eval prompts, the fixtures or the assertions.
- A macOS runner: `bwrap` is Linux-only.
- CI: the run needs a model key and takes minutes.

== Dogfood

This repo runs `just skill-evals --arm both` before it rates `self-describing` again, and records the run id beside the baseline table.
