# Coverage and CRAP

Deep dive behind two lines of CONTRIBUTING.md's gate table: `just cover` and
`just crap`. Read this before changing either recipe's flags.

## Why cargo-llvm-cov

Coverage runs separately from `just check` (`just cover`, CI `coverage` job) via
`cargo-llvm-cov`, which merges each process's profraw — so the subprocess-based
e2e tests credit the command code they exercise. Tarpaulin, which it replaced,
could not see past `exec` and read every command entry point as dead.

The recipe writes `target/coverage/lcov.info` for `just crap` and fails under
90% lines. `main.rs` is excluded — keep it thin.

`just cover` builds default features only, so `recur_rrule.rs` (the `rrule`
cargo feature's expander) never compiles, and `just crap` excludes it too rather
than scoring it as phantom 0%-covered. `just check-rrule` covers it separately,
under the `rrule` feature.

## Why it throws the previous run away

The recipe starts with `cargo llvm-cov clean --workspace`, so it always pays a
full instrumented rebuild (~1 min). Without that, `report` unions every profraw
and object file left since the last `cargo clean` — other feature sets, other
commits — and reports coverage this source cannot produce. It inflated
`crap-baseline.json` once already. Never trade that line for speed. The recipe's
comment carries the measurement.

## The CRAP threshold, and why it is not the whole gate

`just crap` runs right after `just cover`, in the same CI job and over the same
`lcov.info`: the CRAP metric (`CC² × uncovered³ + CC`) fails above 30, flagging
functions that are both complex and undertested. It refuses to run against an
lcov older than a source file, so run `just cover` first — CI already does. Why
it refuses, what it excludes, and why there is deliberately no advisory band
below the threshold are argued in the `crap` recipe's own comment.

One consequence is worth carrying in your head rather than looking up: above
~90% coverage the cubed term vanishes and a function scores its own cyclomatic
complexity, so a flat dispatcher sits near the threshold by construction, and
adding an arm to one means covering it in the same change.

That same collapse is why the threshold alone is not enough. `just crap` runs a
second check against `crap-baseline.json`, a committed score for every function:
nothing may get worse than the baseline, at any absolute level. It never reads
the absolute score, only the delta, so the collapse does not apply — a rise
means complexity went up or coverage went down either way. It is what catches a
function that is complex and undertested but scores well under 30, which the
threshold is structurally blind to.

When you deliberately add complexity, run `just crap-baseline` in the same commit
so the new number lands next to the reason for it.

## Where the tooling comes from

How the dev shell supplies cargo-crap and the LLVM tools `cargo-llvm-cov` needs
is `flake.nix`'s subject — both are non-obvious, both break loudly if changed
carelessly, and both are commented there.
