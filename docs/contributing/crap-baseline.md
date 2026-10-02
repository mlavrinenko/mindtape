<!-- Generated from docs/src/contributing/crap-baseline.typ by `just docs md`. Do not edit; edit the source. -->

# The CRAP baseline

Deep dive behind `just crap-baseline`: what `crap-baseline.json` is, why its paths are repo-relative, and which arm may guard a mint. The ratchet it feeds is argued in [docs/contributing/coverage-crap.md](docs/contributing/coverage-crap.md).

## Why the baseline’s paths are repo-relative

`cargo crap --workspace` reports the member roots `cargo metadata` gives it, which are absolute, and cargo-crap 0.3.1 has no flag to shorten them — so an unedited mint writes the minter’s own machine layout into a tracked file. `just crap-baseline` strips the workspace root back off before writing.

That gives up nothing, because the exact match it looks like it is giving up was never in play. cargo-crap keys its first matching pass on `(file, function, line)`, so a baseline minted under one checkout root already misses that pass entirely under another — CI’s included — and pairs through the suffix-aware pass instead. Relative paths route every machine through the same pass CI has been relying on all along. Measured on the first relative mint: 1056 of 1058 entries paired as unchanged, and the two that did not are one file’s two same-named `fake_editor` functions, ambiguous by suffix and therefore unratcheted under CI’s root already.

`just crap-baseline-check` is what keeps it that way, since a re-mint by hand puts every local path straight back and a thousand-entry JSON diff is not reviewed line by line. It carries a positive control, because its own failure mode is a false green — a filter matching nothing reads exactly like a clean file. gitleaks cannot take that job: measured under `--log-level trace`, the default ruleset’s global allowlist matches `/home/<user>/` and drops any custom rule’s finding on it, and an extending config can only add allowlists, never remove one. gitleaks is in the gate set for the job it is good at — `just check-secrets`, over credential shapes rather than local paths.

## What guards the mint, and what deliberately does not

`just crap-baseline` runs the threshold arm before it writes, never the whole gate. The two arms answer to different owners, and only one of them can guard a mint.

The threshold arm asks whether anything is actually bad. Minting a function over 30 in as acceptable is precisely the freeze a pre-mint guard exists to stop, so that arm aborts the mint — and it refuses a stale lcov on the way, which is what makes the numbers about to be committed ones a clean run reproduces.

The ratchet arm cannot guard it, because it scores against the very file the mint is about to rewrite. A deliberate complexity increase makes that arm red by definition, and a deliberate complexity increase is the one occasion [coverage-crap.md](docs/contributing/coverage-crap.md) tells you to re-mint. Depending on the whole gate would make the recipe refuse in its own documented use case. So the ratchet stays a gate, the threshold arm guards the mint, and `just crap-baseline` needs no manual recovery step.
