#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Ship the skill-eval env schema",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.95,
    ease: 9,
  ),
  difficulty: 1,
  tags: ("tooling", "stateless"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      config-over-code: helps[the model and its key are values in `.env`, not constants in the script],
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
  status: proposed(2026, 11, 20),
  links: related(
    "pick-an-isolated-harness-for-the-skill-evals.typ",
  )[the decision requires the model to come from .env],
)

== AS-IS

+ Read the decision in `pick-an-isolated-harness-for-the-skill-evals.typ`.
+ 🔴 The harness has no place to name its model, so a run would inherit whatever the shell exports.
+ 🔴 `.env` is not ignored, so a key written there lands in the next commit.

```
$ touch .env
$ git status --short .env
?? .env
```

== TO-BE

- 🟢 `.env.schema` lists `SKILL_EVALS_MODEL_ID` and `SKILL_EVALS_API_KEY` with placeholder values, and a comment says to copy it to `.env`.
- 🟢 `.gitignore` ignores `.env`. A filled copy stays out of git.
- 🟢 The placeholders are fake: `example-provider/example-model` and `replace-with-a-real-key`.

```
$ cp .env.schema .env
$ cat .env
SKILL_EVALS_MODEL_ID=example-provider/example-model
SKILL_EVALS_API_KEY=replace-with-a-real-key
```

== Changes

- `.env.schema`: the two variables, their placeholders and the copy instruction.
- `.gitignore`: ignore `.env`.

== Out of scope

- Reading `.env` in the recipe: the harness task does that.
- A `.env` loader for other recipes.

== Dogfood

This repo copies `.env.schema` to `.env` before it runs `just skill-evals`, and the file stays out of git.
