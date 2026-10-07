#!/usr/bin/env bash
# .just/scripts/_mutants-env.sh — the CARGO_MUTANTS_JOBS/RUSTFLAGS preamble
# shared by mutants.sh and mutants-diff.sh. Sourced, not executed on its own
# — an underscore-prefixed helper, same convention as _lcov-stale-reason.sh
# and the `_crap-excludes` recipe.
#
# Three parallel jobs and the mold linker, benchmarked at ~3.5x over the naive
# run. Both are overridable, and both are tuned for the pre-PR pass: drop jobs
# to 2 if parallel load provokes false TIMEOUTs, and see
# docs/contributing/mutation-testing.md for why a sweep whose verdict you
# intend to believe runs at 2 or lower anyway.
#
# Both knobs stay script-local: sourcing this into a script's own process
# exports them for that process and its children (cargo mutants) only, never
# Justfile-wide. RUSTFLAGS in particular must never become a Justfile-level
# `export` — it is part of the cargo fingerprint, so exporting it to every
# recipe would invalidate the build cache for the whole workspace.
#
# The editor pair is the same kind of knob, for the same reason: unit tests
# read `$VISUAL`/`$EDITOR` in-process (`cmd::add::interactive::editor_command`), so a
# mutant that ignores the test's override falls through to whatever editor the
# developer's session carries: a `--wait` GUI editor opens one
# window per mutant, `run_parses_editor_output` killed at the 30 s timeout, the
# mutant recorded as timeout instead of caught. `support::isolate` cannot reach
# an in-process read, so the sweep's own environment is the lock: no `$VISUAL`
# at all, and an `$EDITOR` that cannot exist, so the fall-through is a fast
# spawn error with the test's real assertion still failing.
unset VISUAL
export EDITOR="mt-no-such-editor"
export CARGO_MUTANTS_JOBS="${CARGO_MUTANTS_JOBS:-3}"
export RUSTFLAGS="${RUSTFLAGS:-} -Clink-arg=-fuse-ld=mold"

# Each mutant builds and tests the whole workspace (`test_workspace` in
# .cargo/mutants.toml), and Cargo's default build parallelism is one rustc per
# core *per mutant*. Two mutant workers put ~14 concurrent `mindtape` rustc
# processes in flight at up to ~6 GB each, enough to exhaust memory and get
# the sweep killed mid-run. Cap each
# build's internal parallelism so a bounded number of rustc processes is ever
# live; `CARGO_BUILD_JOBS` overrides it. `just test` is untouched.
export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-2}"

# No kache. `RUSTC_WRAPPER=` empty is read by cargo as no wrapper and beats
# config and the dev shell's value. Every mutant edits `crates/mindtape`, so it
# and its dependents re-key and can never hit; only registry deps can, and cargo
# reuses those per scratch copy after the first build. The 2026-09-30 sweep
# paid ~28 min keying and storing 1326 misses plus ~25 min keying 2442
# passthroughs for ~400 s of hits, and wrote 146.5 GB through the 60 GB store,
# evicting what normal builds hit. Measured here 2026-10-01 on restricted
# sweeps, with kache vs without: 20 mutants 704 s vs 772 s (+833 store entries
# vs 0; the loss is the cold registry-dep build in each of three scratch dirs,
# 99 s vs 168 s baseline build, a one-off per job), 81 mutants 1337 s vs 1078 s
# (+1184 entries vs 0). Steady-state build per mutant ~70 s vs ~46 s.
export RUSTC_WRAPPER=

# A sweep costs about 13 s a mutant (4240 took 15 h on 2026-09-30), so a run
# over MUTANTS_BUDGET mutants waits for the owner. Only the owner sets
# MINDTAPE_MUTANTS_APPROVED=1; an agent stops and asks. `cargo mutants --list`
# takes the same selection args as the run, so the count is the run's.
MUTANTS_BUDGET=100
mutants_budget_check() {
    local count
    count=$(cargo mutants --list "$@" | wc -l)
    if [ "$count" -le "$MUTANTS_BUDGET" ] || [ "${MINDTAPE_MUTANTS_APPROVED:-}" = 1 ]; then
        return 0
    fi
    echo "error: this run would test $count mutants, about $((count * 13 / 3600)) h at 13 s each" >&2
    echo "hint: a run over $MUTANTS_BUDGET mutants needs the owner's approval: MINDTAPE_MUTANTS_APPROVED=1 just $(basename "$0" .sh)" >&2
    echo "hint: or narrow it, for example with -f <file>" >&2
    exit 1
}
