#!/usr/bin/env bash
# .just/scripts/check-gate-drift.sh — the body of the `check-gate-drift` recipe
# (truth/tasks/fail-the-gate-when-toml-fmt-paths-or-a-plain-mt-build-drifts.typ).
# Runs from the repo root, like the recipe that invokes it.
#
# Two facts the gate depends on are kept by hand, and each failed silently
# once. This arm reads both from their source and names the offender.
#
#   1. TOML_FMT_PATHS (Justfile) is the list `just fmt-check` and `just fmt`
#      hand to tombi. A TOML file missing from it is never format-checked; a
#      path in it that git does not know makes tombi fail on a file that is
#      gone. The reference is what git knows — `git ls-files --cached --others
#      --exclude-standard '*.toml'`, so a new file not yet `git add`ed counts
#      the moment it exists, and an ignored one never does. The value comes out
#      of `just --evaluate`, not a grep of the Justfile, so a rewrite of how the
#      variable is built cannot blind the arm. Both directions are reported.
#   2. `target/debug/mt` is one path for two builds (32d79fd9, and
#      docs/contributing/gates.md "One mt binary, one feature set"): the test
#      arm links it with mindtape-cli's dev-dependency features, so a plain
#      `cargo build -p mindtape-cli` or `cargo run -p mindtape-cli` anywhere a
#      gate reaches swaps the link under the running tests. Every build there
#      has to carry `--tests`.
#
# WHAT ARM TWO READS: only what a `just check` arm reaches, as _script-graph.sh
# walks it (`_script-reach.sh ... check`): the recipe names `check` hands to `mmz`, recipe
# dependencies, a nested `just <recipe>`, and any script a body or script
# names, with www/*.sh and docs/*.sh added as nodes. The walk is the one
# script-scope-check already trusts, so a shell form it learns, this arm learns;
# nothing is listed here, and a script that arrives is read by arriving. It
# reads a script NAMED as much as one run, so it errs toward scanning more.
#
# `just check-rrule` is OUT of scope, and by that rule rather than by name: it
# is not a `just check` arm, so nothing here reaches it. It builds with
# `cargo run --features rrule` into the shared target/debug on purpose (the
# feature set is the point, and `cargo run` takes no `--tests`), and CI runs it
# as its own step after `just check` has finished, serially, after its own
# nextest has stopped using the binary. The race this arm guards needs a test
# run in flight beside the build; `check-rrule` has none. If it ever becomes an
# arm of `check`, it becomes reachable and fails here until it gets another
# `--target-dir`, which is the right outcome.
#
# EXEMPT BY FLAG, never by file: `--release` (`-r`) and `--target-dir` (or a
# `CARGO_TARGET_DIR=` prefix) write somewhere else, `--tests` resolves the test
# arm's unit, and `-p <other>` builds another crate. `cargo run` is never
# exempted by `--tests` — it takes none — only by writing elsewhere. A cargo
# with no `-p`, or with `--workspace`, builds mindtape-cli too.
#
# CONTROL FIRST (_gate-drift-control.sh), for the reason check-secrets.sh gives for its own: "no
# findings" is equally what a reader that stopped following edges prints. The
# control builds synthetic trees in a temp dir (never the working tree: `just
# check` runs its arms in parallel, so a probe there would fail a concurrent
# gate), requires the clean one to pass, and requires the bad ones to fail with
# exactly the offenders named.
#
# The readers are in _gate-drift-lib.sh (sourced here and by the control), the
# control in _gate-drift-control.sh; this file is the argument and the entry.
#
# Static: no recipe body runs, nothing is built.
set -eo pipefail

# shellcheck source=.just/scripts/_gate-drift-lib.sh
source .just/scripts/_gate-drift-lib.sh

bash .just/scripts/_gate-drift-control.sh

status=0
toml_drift . || status=1
build_drift . || status=1
exit $status
