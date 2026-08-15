<!-- Generated from docs/src/contributing/coverage-crap.typ by `just docs-md`. Do not edit; edit the source. -->

# Coverage and CRAP

Deep dive behind two lines of CONTRIBUTING.md’s gate table: `just cover` and `just crap`. Read this before changing either recipe’s flags.

## Why cargo-llvm-cov

Coverage runs separately from `just check` (`just cover`, CI `coverage` job) via `cargo-llvm-cov`, which merges each process’s profraw — so the subprocess-based e2e tests credit the command code they exercise. Tarpaulin, which it replaced, could not see past `exec` and read every command entry point as dead.

The recipe writes `target/coverage/lcov.info` for `just crap` and fails under 90% lines. `main.rs` is excluded — keep it thin.

`just cover` builds default features only, so `recur_rrule.rs` (the `rrule` cargo feature’s expander) never compiles, and `just crap` excludes it too rather than scoring it as phantom 0%-covered. `just check-rrule` covers it separately, under the `rrule` feature.

## Why it throws the previous run away

`just cover` starts with `cargo llvm-cov clean --workspace` — in `.just/scripts/cover.sh`, where the recipe’s body lives — so it always pays a full instrumented rebuild (~1 min). Without that, `report` unions every profraw and object file left since the last `cargo clean` — other feature sets, other commits — and reports coverage this source cannot produce. It inflated `crap-baseline.json` once already. Never trade that line for speed.

That line is also cargo-llvm-cov’s own documented precondition for the `--no-report` / `report` split (“should be called before the first run”): `--no-report` deliberately does not clear the profraw pile, because the split exists so several runs CAN be merged — the tool’s own recipe for coverage across feature sets. What it costs here is a rebuild; what it buys is a number that means something.

Measured, not theorised (`truth/tasks/clean-the-profraw-pile-before-each-coverage-run.typ`): two runs left 461+461 profraw; merging one `--features rrule` run took `check_rrule_rule` from 57% to 100%, coverage of a branch that is dead code without the feature; and the pile that accumulated across a whole session put `ls_interactive::apply` at 84% when no test in the tree can call it at all. That lcov minted `crap-baseline.json`, so the ratchet then failed 48 functions on the first honest run after a `cargo clean`.

Inflated coverage is the worst failure this repo can have here — it hides exactly what `just crap` and the sweep’s coverage arms exist to find — so pay the instrumented rebuild.

## The CRAP threshold, and why it is not the whole gate

`just crap` runs right after `just cover`, in the same CI job and over the same `lcov.info`: the CRAP metric (`CC² × uncovered³ + CC`) fails above 30, flagging functions that are both complex and undertested. It refuses to run against an lcov older than a source file, so run `just cover` first — CI already does. Why it refuses, and why there is deliberately no advisory band below the threshold, are the two sections after this one; what it excludes, and why each exclude flag has to be spelled `**/…`, are argued in place in `.just/scripts/crap.sh`.

One consequence is worth carrying in your head rather than looking up: above ~90% coverage the cubed term vanishes and a function scores its own cyclomatic complexity, so a flat dispatcher sits near the threshold by construction, and adding an arm to one means covering it in the same change.

That same collapse is why the threshold alone is not enough. `just crap` runs a second check against `crap-baseline.json`, a committed score for every function: nothing may get worse than the baseline, at any absolute level. It never reads the absolute score, only the delta, so the collapse does not apply — a rise means complexity went up or coverage went down either way. It is what catches a function that is complex and undertested but scores well under 30, which the threshold is structurally blind to.

When you deliberately add complexity, run `just crap-baseline` in the same commit so the new number lands next to the reason for it.

## Why a stale lcov is refused rather than scored against

The lcov is a build artifact nothing re-derives for you, and scoring against a stale one does not degrade gracefully: a function in a file written since the lcov has no coverage record at all, `--missing` defaults to `pessimistic`, and so it reads as 0% covered rather than as unknown. Observed on a 10-day-old lcov — 28 failures, 27 of them phantom (each showed coverage `—`, not a percentage).

A gate that invents failures is worse than no gate, so `just crap` refuses a stale lcov and names the recipe that refreshes it. CI is unaffected: its `coverage` job runs `just cover` immediately before it.

## Why there is no advisory band below the threshold

There is deliberately no advisory band below this gate, and it was measured rather than assumed (`truth/tasks/sweep-crap-scores-into-tasks.typ`). CRAP is `CC^2 * (1-cov)^3 + CC`, so above ~90% coverage the cubed term vanishes and the score IS cyclomatic complexity: of the 15 functions scoring 15..30 here, 11 are at or above 90% covered, while 12 of the 16 functions that are genuinely complex and undertested score below 15. A band below 30 would be wrong in both directions at once, so `just ask-refactoring` has no CRAP arm.

## The check-crap arm, and what it does not buy

`just check-crap` is the CRAP gate as a `just check` arm. It runs the real gate when the lcov is fresh; otherwise it skips loudly, naming what would refresh it, and exits 0.

Why it exists: `just crap` is CI-enforced and used to be absent from `just check`, so nothing local ever evaluated it and a green `just check` carried no information about it. That is how it sat red on main (`truth/tasks/cover-confirm-retire-and-unred-the-crap-gate.typ`).

Be honest about what this buys. Editing any source stales the lcov, so on precisely the commits that matter this arm SKIPS. It catches the free case — a check run after a recent `just cover` — and nothing more. `just check-full` is the arm that is always correct; this one is the cheap partial. It follows the signal sweep’s mutants arm: absent and clean must never look alike, so it says which of the two it is every time.

Why it is the one `just check` arm that is not memoized is in [docs/contributing/gates.md](docs/contributing/gates.md).

## When the baseline itself is the thing that is wrong

`just crap-baseline` depends on `just crap`, so a red gate aborts the mint: the baseline records today as acceptable, and minting while something is red freezes the red. That guard has one blind spot, and it has fired once. If the BASELINE is what is wrong, `just crap` fails forever and the recipe refuses to refresh the file that would fix it. It happens when the baseline was minted over an lcov that no clean run reproduces — before `just cover` cleaned the profraw pile, a mint could credit coverage from another feature set or another commit.

Recovery is deliberately manual: one commit that re-mints against a fresh lcov, carrying the two facts that make it not a bypass in its message. First, `cargo crap --lcov ... --threshold 30 --min 30 --fail-above` is green — nothing is actually bad. Second, no cyclomatic complexity moved: the ratchet’s real subject is unchanged, only the measurement was. Run the recipe body by hand.

## Where the tooling comes from

How the dev shell supplies cargo-crap and the LLVM tools `cargo-llvm-cov` needs is `flake.nix`’s subject — both are non-obvious, both break loudly if changed carelessly, and both are commented there.
