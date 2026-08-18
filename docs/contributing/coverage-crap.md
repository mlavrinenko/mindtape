<!-- Generated from docs/src/contributing/coverage-crap.typ by `just docs-md`. Do not edit; edit the source. -->

# Coverage and CRAP

Deep dive behind two lines of CONTRIBUTING.md’s gate table: `just cover` and `just crap`. Read this before changing either recipe’s flags.

## Why cargo-llvm-cov

Coverage runs separately from `just check` (`just cover`, CI `coverage` job) via `cargo-llvm-cov`, which merges each process’s profraw — so the subprocess-based e2e tests credit the command code they exercise. Tarpaulin, which it replaced, could not see past `exec` and read every command entry point as dead.

The recipe writes `target/coverage/lcov.info` for `just crap` and fails under 90% lines. `main.rs` is excluded — keep it thin.

`just cover` builds default features only, so `recur_rrule.rs` (the `rrule` feature’s expander) never compiles, and `just crap` excludes it too rather than scoring it as phantom 0%-covered. `just check-rrule` covers it separately, under the `rrule` feature.

## Why it throws the previous run away

`just cover` starts with `cargo llvm-cov clean --workspace` — in `.just/scripts/cover.sh`, where the recipe’s body lives — so it always pays a full instrumented rebuild (~1 min). Without it, `report` unions every profraw and object file left since the last `cargo clean` — other feature sets, other commits — and reports coverage this source cannot produce. Never trade that line for speed.

The pile is not an accident: `--no-report` deliberately keeps it so several runs CAN be merged across feature sets, and cargo-llvm-cov documents the clean as that split’s precondition (“should be called before the first run”).

What the pile costs, measured (`truth/tasks/clean-the-profraw-pile-before-each-coverage-run.typ`): two runs left 461+461 profraw; merging one `--features rrule` run took `check_rrule_rule` from 57% to 100%, coverage of a branch that is dead code without the feature; and a whole session’s pile put `ls_interactive::apply` at 84% when no test in the tree can call it at all. That lcov minted `crap-baseline.json`, so the ratchet then failed 48 functions on the first honest run after a `cargo clean`.

Inflated coverage is the worst failure this repo can have here — it hides exactly what `just crap` and the sweep’s coverage arms exist to find.

## The CRAP threshold, and why it is not the whole gate

`just crap` runs right after `just cover`, in the same CI job and over the same `lcov.info`. The CRAP metric is `CC² × uncovered³ + CC`, and the gate fails above 30, flagging functions that are both complex and undertested. It refuses an lcov older than a source file, so run `just cover` first — CI already does. What it excludes, and why each exclude flag has to be spelled `**/…`, are argued in place in `.just/scripts/crap.sh`.

Carry one consequence of that formula in your head: above ~90% coverage the cubed term vanishes and a function scores its own cyclomatic complexity. A flat dispatcher therefore sits near the threshold by construction, and adding an arm to one means covering it in the same change.

That collapse is why the threshold alone is not enough. `just crap` runs a second check against `crap-baseline.json`, a committed score for every function: nothing may get worse than the baseline, at any absolute level. It reads only the delta, so the collapse does not apply — a rise means complexity went up or coverage went down either way — and it catches the function that is complex and undertested but scores well under 30, which the threshold is structurally blind to.

When you deliberately add complexity, run `just crap-baseline` in the same commit so the new number lands next to the reason for it.

## Why a stale lcov is refused rather than scored against

Scoring a stale lcov does not degrade gracefully: a function in a file written since the lcov has no coverage record at all, `--missing` defaults to `pessimistic`, and so it reads as 0% covered rather than as unknown. Observed on a 10-day-old lcov — 28 failures, 27 of them phantom (each showed coverage `—`, not a percentage).

A gate that invents failures is worse than no gate, so `just crap` refuses a stale lcov and names the recipe that refreshes it. CI is unaffected: its `coverage` job runs `just cover` immediately before it.

## Why there is no advisory band below the threshold

The absent band was measured, not assumed (`truth/tasks/sweep-crap-scores-into-tasks.typ`). Because of the collapse above, of the 15 functions scoring 15..30 here, 11 are at or above 90% covered, while 12 of the 16 genuinely complex and undertested ones score below 15. A band below 30 would be wrong in both directions at once, so `just ask-refactoring` has no CRAP arm.

## The check-crap arm, and what it does not buy

`just check-crap` is the CRAP gate as a `just check` arm. It runs the real gate when the lcov is fresh; otherwise it skips loudly, naming what would refresh it, and exits 0.

It exists because `just crap` is CI-enforced and used to be absent from `just check`, so nothing local ever evaluated it and a green `just check` carried no information about it. That is how it sat red on main (`truth/tasks/cover-confirm-retire-and-unred-the-crap-gate.typ`).

Be honest about what it buys. Editing any source stales the lcov, so on precisely the commits that matter this arm SKIPS: it catches the free case, a check run after a recent `just cover`, and nothing more. `just check-full` is the arm that is always correct. Like the signal sweep’s mutants arm, it says which of the two it is every time — absent and clean must never look alike.

It is memoized like every other `just check` arm, over an `lcov` input scope naming the artifact it reads, so a recorded skip dies the moment that artifact appears. What that scope costs and why it needs its own `gitignore` opt-out is in [docs/contributing/gates.md](docs/contributing/gates.md).

## Why the baseline’s paths are repo-relative

`cargo crap --workspace` reports the member roots `cargo metadata` gives it, which are absolute, and cargo-crap 0.3.1 has no flag to shorten them — so an unedited mint writes the minter’s own machine layout into a tracked file. `just crap-baseline` strips the workspace root back off before writing.

That gives up nothing, because the exact match it looks like it is giving up was never in play. cargo-crap keys its first matching pass on `(file, function, line)`, so a baseline minted under one checkout root already misses that pass entirely under another — CI’s included — and pairs through the suffix-aware pass instead. Relative paths route every machine through the same pass CI has been relying on all along. Measured on the first relative mint: 1056 of 1058 entries paired as unchanged, and the two that did not are one file’s two same-named `fake_editor` functions, ambiguous by suffix and therefore unratcheted under CI’s root already.

`just crap-baseline-check` is what keeps it that way, since a re-mint by hand puts every local path straight back and a thousand-entry JSON diff is not reviewed line by line. It carries a positive control, because its own failure mode is a false green — a filter matching nothing reads exactly like a clean file. gitleaks cannot take that job: measured under `--log-level trace`, the default ruleset’s global allowlist matches `/home/<user>/` and drops any custom rule’s finding on it, and an extending config can only add allowlists, never remove one. gitleaks is in the gate set for the job it is good at — `just check-secrets`, over credential shapes rather than local paths.

## When the baseline itself is the thing that is wrong

`just crap-baseline` depends on `just crap`, so a red gate aborts the mint — the baseline records today as acceptable, and minting while something is red would freeze the red. The one blind spot, which has fired once: if the BASELINE is what is wrong, `just crap` fails forever and the recipe refuses to refresh the file that would fix it. That happens when the baseline was minted over an lcov no clean run reproduces, as one could before `just cover` cleaned the profraw pile.

Recovery is deliberately manual: run the recipe body by hand, in one commit that re-mints against a fresh lcov and carries the two facts that make it not a bypass. First, `cargo crap --lcov ... --threshold 30 --min 30 --fail-above` is green — nothing is actually bad. Second, no cyclomatic complexity moved: the ratchet’s real subject is unchanged, only the measurement was.

## Where the tooling comes from

How the dev shell supplies cargo-crap and the LLVM tools `cargo-llvm-cov` needs is `flake.nix`’s subject — both are non-obvious, both break loudly if changed carelessly, and both are commented there.
