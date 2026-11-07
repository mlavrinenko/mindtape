# mindtape skill evals

Six evals in `evals.json`, run through skill-creator's loop: spawn an executor
subagent per eval twice, once with the skill and once with no external
documentation, then grade each run against its own block. `assertions` grades
the arm with the skill. `without.assertions` grades a consumer given only the
prompt and `mt`. The delta is what the skill adds, and a no-docs failure names
a fact the CLI does not carry yet.

## Fixtures

Evals run against the tracked example projects, not a fixture of their own:

| Fixture | Evals | Why this one |
|---------|-------|--------------|
| `examples/api-service` | 0-3 | 14 tasks, 4 of them done, ICE priority with bands, a default template with required slots, a milestone template whose goal ratings a `check` rule reads, a cap of one milestone and one other task in `wip`, a `proposed` task blocked by a dependency, another whose `waits-on` checker (`scripts/locked`) `mt` runs only after `mt allow`, which the agent must leave to the user, and a `scripts/test` gate on every `mt done`, which `mt` likewise holds back until `mt allow` and which fails once allowed |
| `examples/minimal` | 4-5 | The smallest config, with its own four-status legend (`todo`, `wip`, `done`, `cancelled`). Its two absences are the test: no `blocked` in the legend, and no template, so `--set priority=8` is refused by name |

Both are validated by `just check-examples`, which is what makes them safe to
depend on. The previous fixture lived in a gitignored scratch tree with no gate
and silently outlived two config migrations. Every eval would have died at
config load, and nothing said so.

## Running one

Copy the fixture to a scratch dir and run there. Never in place: evals mutate
(`mt add` writes files, `mt flip` restamps) and these are tracked, gated
fixtures. A stray write shows up as a `just check` failure later, in a commit
that has nothing to do with it.

The fixture's config points `@local/mindtape` at `../../lib`, so the copy has
to sit two levels below the repo root. `target/` is the gitignored place that
depth allows; a copy under `/tmp` breaks the path at config load.

```sh
mkdir -p target
work=$(mktemp -d -p target)
cp -a examples/api-service/. "$work/"
# hand the subagent "$work" as its cwd
```

Set `MINDTAPE_NOW=2026-10-01` for `examples/api-service`, the clock
`www/capture.sh` declares and the evals and their e2e test read. Otherwise a
due date or a start date moves against the wall clock and the expected output
stops matching.

Symlinking the fixture instead does not work: `cp` without `-L` leaves a
dangling link in the sandbox, and following it lets the eval write into the
tracked example.

`evals.json` carries a `fixture` key per eval with the repo-relative path. The
schema's own `files` array is for input documents relative to the skill root,
so it stays empty here. A whole project tree that must be copied is a different
thing, and overloading `files` would hand the orchestrator a path that resolves
nowhere.

## Two arms

The with-skill arm loads `skills/mindtape/` and grades `assertions`. The
no-docs arm gets the prompt, the scratch copy and `mt`, and grades
`without.assertions`; it reads no skill, no page under `docs/`, and nothing
from the `www/` site. Record each arm's pass and fail, and file a task for a
failed no-docs assertion naming the CLI fact it needed.

`just skill-evals-check` validates that every eval carries both blocks and that
every fixture path exists. It reads the manifest and the filesystem, so it runs
in `just check` without a model.

## Model

Run the executor and grader subagents on a mid-tier model, not the strongest
one. These evals check whether the skill supplies a fact the agent would
otherwise lack (that `--all` does not exist, that an unknown `--set` key is
refused by name, that a legend may have no `blocked`) and that is not a
reasoning-depth question. A stronger model partly masks the gap by recovering
from the error message, which makes the with-skill and no-docs runs converge
and the eval less discriminating. A mid-tier model also keeps a six-eval double
run affordable enough to actually repeat.

Grade on the same model for both arms of a comparison, or the delta measures
the grader instead of the skill.

## Baseline

Recorded 2026-11-07 for eval 0 (`ready-priority-triage`) on
`examples/api-service`, with no external documentation. The executor was the
agent that wrote this file, and it had read `evals.json` first, so the run is a
reproduction, not a blind sample.

The trace used only the prompt and `mt`, with the fixture clock set:

```sh
mt ls        # default view, 9 of 14 tasks: 7 ready and 2 blocked
mt ls ready  # 8 of 14 tasks, both blocked tasks gone
```

`mt ls` sorts by priority and its `READY` column marks the two blocked tasks,
so the reader saw "Index the orders by customer" first without a filter.
`mt ls ready` then dropped "Throttle failed logins per IP" (unfinished
dependency) and "Verify webhooks with the acme-pay SDK" (checker not allowed).
The warning named `scripts/locked {id}` and `scripts/test`, and the trace ran
no `mt allow`. All seven `without.assertions` passed. No CLI fact was missing,
so there is no failure to name. Eval 0 needs no skill fact.

Evals 1-5 remain unrun in the no-docs arm. The first release that runs both
arms records their results here.
