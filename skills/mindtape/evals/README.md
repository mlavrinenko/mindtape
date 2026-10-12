# mindtape skill evals

Six evals in `evals.json`, run through skill-creator's loop (spawn an executor
subagent per eval with and without the skill, grade each assertion, aggregate).

## Fixtures

Evals run against the tracked example projects, not a fixture of their own:

| Fixture | Evals | Why this one |
|---------|-------|--------------|
| `examples/api-service` | 0-3 | 14 tasks, 4 of them done, ICE priority with bands, a template with required slots, a cap of one milestone and one other task in `wip`, a `proposed` task blocked by a dependency, another whose `waits-on` checker (`scripts/locked`) `mt` runs only after `mt allow`, which the agent must leave to the user, and a `scripts/test` gate on every `mt done`, which `mt` likewise holds back until `mt allow` and which fails once allowed |
| `examples/minimal` | 4-5 | The floor config. Its two absences are the test: no `blocked` in the legend, and no template — so `--set priority=8` is refused by name |

Both are validated by `just check-examples`, which is what makes them safe to
depend on. The previous fixture lived in a gitignored scratch tree with no gate
and silently outlived two config migrations — every eval would have died at
config load, and nothing said so.

## Running one

Copy the fixture to a scratch dir and run there. Never in place: evals mutate
(`mt add` writes files, `mt flip` restamps) and these are tracked, gated
fixtures — a stray write shows up as a `just check` failure later, in a commit
that has nothing to do with it.

```sh
work=$(mktemp -d)
cp -a examples/api-service "$work/project"
# hand the subagent "$work/project" as its cwd
```

Symlinking the fixture instead does not work: `cp` without `-L` leaves a
dangling link in the sandbox, and following it lets the eval write into the
tracked example.

`evals.json` carries a `fixture` key per eval with the repo-relative path. The
schema's own `files` array is for input documents relative to the skill root, so
it stays empty here — a whole project tree that must be copied is a different
thing, and overloading `files` would hand the orchestrator a path that resolves
nowhere.

## Model

Run the executor and grader subagents on a mid-tier model, not the strongest
one. These evals check whether the
skill supplies a fact the agent would otherwise lack — that `--all` does not
exist, that an unknown `--set` key is refused by name, that a legend may have no
`blocked` — and that is not a reasoning-depth question. A stronger model partly masks the
gap by recovering from the error message, which makes the with-skill and
without-skill runs converge and the eval less discriminating. A mid-tier model
also keeps a six-eval double run affordable enough to actually repeat.

Grade on the same model for both arms of a comparison, or the delta measures the
grader instead of the skill.
