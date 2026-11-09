# mindtape skill evals

Six evals in `evals.json`, run through skill-creator's loop: spawn an executor
subagent per eval twice, once with the skill and once with no external
documentation, then grade each run against its own block. `assertions` grades
the arm with the skill. `without.assertions` grades a consumer given only the
prompt and `mt`. The difference between the arms shows what the skill adds. A
no-docs failure identifies a fact the CLI does not yet expose.

## Fixtures

Evals run against the tracked example projects, not a fixture of their own:

| Fixture | Evals |
|---------|-------|
| `examples/api-service` | 0-3 |
| `examples/minimal` | 4-5 |

`examples/api-service` has 14 tasks, 4 of them done:

- ICE priority with bands.
- A default template with required slots.
- A milestone template whose goal ratings a `check` rule reads.
- A cap of one milestone and one other task in `wip`.
- A `proposed` task blocked by a dependency.
- Another task whose `waits-on` checker (`scripts/locked`) `mt` runs only after
  `mt allow`, which the agent must leave to the user.
- A `scripts/test` gate on every `mt done`, which `mt` likewise holds back
  until `mt allow` and which fails once allowed.

`examples/minimal` is the smallest config, with its own four-status legend
(`todo`, `wip`, `done`, `cancelled`). Its two absences are the test: no
`blocked` in the legend, and no template, so `--set priority=8` is refused by
name.

`just check-examples` validates both projects, so a config change cannot leave
them stale. An unvalidated fixture can drift through several config changes and
make every eval fail at config load without any check reporting it.

## Running one

Copy the fixture to a scratch directory and run there. Do not run in place.
These are tracked, gated fixtures; `mt add` writes files and `mt flip` restamps,
so a stray write fails `just check` in a later commit.

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

`evals.json` has a `fixture` key per eval with the repo-relative path. The
schema's own `files` array is for input documents relative to the skill root,
so it stays empty here. A whole project tree that must be copied is a different
thing, and overloading `files` would give the orchestrator a path that resolves
nowhere.

## Two arms

The with-skill arm loads `skills/mindtape/` and grades `assertions`. The
no-docs arm gets the prompt, the scratch copy and `mt`, and grades
`without.assertions`; it reads no skill, no page under `docs/`, and nothing
from the `www/` site. Record each arm's pass and fail, and file a task for a
failed no-docs assertion naming the CLI fact it needed.

`just skill-evals-check` validates that every eval has both blocks and that
every fixture path exists. It reads the manifest and the filesystem, so it runs
in `just check` without a model.

## Model

Run both subagents on a mid-tier model. These evals check whether the skill
supplies a fact the agent would otherwise lack (that `--all` does not exist,
that an unknown `--set` key is refused by name, that a legend may have no
`blocked`). They do not test reasoning depth. A stronger model partly masks the
gap by recovering from the error message, which makes the with-skill and
no-docs runs converge and the eval less discriminating. A mid-tier model also
keeps a six-eval double run affordable enough to repeat.

Grade on the same model for both arms of a comparison, or the delta measures
the grader instead of the skill.

## Baseline

Recorded 2026-11-07 against scratch copies of `examples/api-service` and
`examples/minimal` under `target/`, with `MINDTAPE_NOW=2026-10-01` and only the
eval prompt and `mt`. Evals 1-5 ran in fresh executor subagents with no skill
and no external documentation. Eval 0 is a reproduction: its executor was the
agent that wrote this file and had read `evals.json` first, so treat it as a
shape check rather than a blind sample.

These blind evals support the `self-describing` goal in `truth/goals.typ`.
They list, file, flip and record a blocker from `mt` alone, so a later no-docs
run that cannot finish a task without reading a page is evidence against that
goal.

| Eval | Result | Arm |
|------|--------|-----|
| 0 `ready-priority-triage` | 7/7 | reproduction |
| 1 `file-new-task` | 4/4 | blind |
| 2 `mark-task-done` | 6/6 | blind |
| 3 `list-everything-no-all-flag` | 4/4 | blind |
| 4 `add-with-priority-slot-trap` | 6/6 | blind, assertion widened after the run |
| 5 `external-blocker-no-blocked-status` | 5/5 | blind |

**Eval 0.** `mt ls` printed the default view, 9 of 14 tasks, with the `READY`
column marking two blocked, so "Index the orders by customer" led without a
filter. `mt ls ready` printed 8 of 14, dropping "Throttle failed logins per IP"
(unfinished dependency) and "Verify webhooks with the acme-pay SDK" (checker
not allowed). The warning named `scripts/locked {id}` and `scripts/test`, and
the trace ran no `mt allow`. All seven assertions passed, so no CLI fact was
missing.

**Eval 1.** The run filed
`fix-password-reset-links-returning-404-in-production.typ` with
`mt add --values ... --no-verify --format json`, after `mt add` refused the
default template's unknown `kind` and `environment` slots. `mt set kind=fix`
then refused until an Environment section existed; the executor added one by
hand and the retry succeeded. `mt check` printed `ok: 15 tasks` and the 14
original task files stayed unchanged. The CLI fact the run needed is the
`mt set` refusal that names the missing section: the default `task` template
has no `kind` or `environment`, and `--no-verify` lets the first write
through, so a consumer only learns the shape from that refusal. This is
friction in the CLI, not an assertion failure; all four assertions passed.

**Eval 2.** `mt done qre -m "..."` printed `skipped: scripts/test (not
allowed)` and `error: gate not allowed: 'scripts/test'`, leaving the task
`wip`. The executor ran neither `mt allow` nor `--force`, reported the gate to
the user, and changed no other task. All six assertions passed.

**Eval 3.** `mt ls --help` mentioned the empty filter. `mt ls ''` listed all 14
tasks, including the four done ones, and the executor did not try `--all`. No
task file changed, and all four assertions passed.

**Eval 4.** The executor read `mt add --describe`, saw the built-in template's
only slot is `title`, filed `upgrade-the-postgres-driver`, and ran
`mt set upgrade-the-postgres-driver priority=8`. The task has `priority: 8`,
`mt check` printed `ok: 6 tasks`, and the five original task files stayed
unchanged. It did not exercise `--set priority=8`, so the original
path-specific assertion failed. The CLI refusal was verified separately:
`mt add "Probe" --set priority=8` exits non-zero with
`error: unknown slot: 'priority'` and writes nothing. The assertion accepts either
path, because the property is that the slot is not silently dropped.
All six pass.

**Eval 5.** `mt flip --list` showed the legend has no `blocked` status. The
executor filed `upstream-design-system-release.typ` and linked `add-dark-mode`
to it with `depends-on("upstream-design-system-release.typ")[the release has not
shipped yet]`. `mt check` printed `ok: 6 tasks`, `mt ls` marks `add-dark-mode`
blocked, and no status was invented. All five assertions passed.
