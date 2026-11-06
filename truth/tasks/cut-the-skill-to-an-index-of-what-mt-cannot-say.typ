#import "@local/mindtape:0.4.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Cut the skill to an index of what `mt` cannot say",
  kind: "docs",
  priority: ice(
    impact: 7,
    confidence: 0.9,
    ease: 5,
  ),
  difficulty: 2,
  tags: ("docs", "stateless"),
  x-goals: (
    product: verdicts(
      (
        cheap-adoption: helps[a smaller skill is cheaper to install and keep current],
      ),
      ..(
        "readable-tasks",
        "implementable-tasks",
        "enforced-practice",
        "typst-over-markdown",
        "plain-files",
      ),
    ),
    process: verdicts(
      (
        docs-match-code: helps[the skill stops restating facts the CLI and its tests already own],
      ),
      ..("sound-codebase",),
    ),
  ),
  assignee: "agent",
  status: done(2026, 11, 7),
)

== AS-IS

+ Open `skills/mindtape/SKILL.md` and compare it with `mt --help` and `mt explain`.
+ 🔴 The skill repeats the command table, filter syntax and filing mechanics those commands already carry, so a CLI change makes it stale twice.

```
$ rg -n '^\| ' skills/mindtape/SKILL.md
36:| List the default view | `mt ls` |
37:| List every task | `mt ls ''` (there is no `--all`) |
38:| Filter and sort | `mt ls '<filter>' --sort <field>:desc` |
```


== TO-BE

- 🟢 `SKILL.md` opens with the trigger, the `.mindtape` check and `mt explain` first.
- 🟢 It keeps only what no command prints: the safety invariants around `mt allow` and `--force`, and where the references live.
- 🟢 Every workflow detail is one hop away in `mt <cmd> --help`, `mt explain` or a reference.
- 🟢 The rendered skill stays under 30 lines.

```
$ wc -l skills/mindtape/SKILL.md
28 skills/mindtape/SKILL.md
```


== Source of truth

- `crates/mindtape-cli/src/cli/mod.rs`: the commands and flags `mt --help` prints.
- `crates/mindtape-cli/src/cmd/explain/index.rs`: what `mt explain` prints.


== Changes

- `docs/src/skill.typ`: cut the command table and the duplicated mechanics; keep the trigger, project check, `mt explain` first and safety invariants.
- `skills/mindtape/SKILL.md`: regenerate from the source with `just docs md`.
- `skills/mindtape/references/*.md`: move a fact that only the skill carries into a reference, then delete the duplicate.
- `www/content/agents.typ`: say the skill points at the CLI instead of repeating it.


== Out of scope

- The CLI messages, which `name-the-next-command-in-every-mt-dead-end` owns.
- The eval expectations, which `grade-the-arm-with-no-external-documentation-in-the-skill-evals` owns.


== Dogfood

Agents working this repo load the cut skill, and `just docs md` plus `just check` regenerate and lint it.

