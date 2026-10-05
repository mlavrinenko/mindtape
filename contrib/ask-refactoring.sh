#!/usr/bin/env bash
# ask-refactoring.sh — turn advisory quality signals into backlog tasks.
#
# Run it as `just ask-refactoring` (dry) or `just ask-refactoring --write`.
#
# Mechanical refactoring work recurs, and today a human notices it at the moment
# it blocks something else. The tools that predict it are already
# machine-readable; nothing reads them into the backlog. This does.
#
# Three rules the design turns on:
#
#   Only advisory signals qualify. A signal that already blocks a commit —
#   jscpd, outdatty, machete, clippy — must never file a task, because the wall
#   is already in front of you and the task would be a note about the wall.
#
#   One task per finding, never one per sweep. Each is dispatched to its own
#   agent and has to carry its own scope.
#
#   Idempotency comes from the signal, not from a ledger. Nothing here records
#   what was filed. A closed task whose condition still holds should be noticed
#   again — the work did not happen. The one deliberate suppression is an
#   override with a written reason in the signal's own config (.linecop.yaml for
#   sizes; an `--exclude` entry in the `just cover`/`just crap` recipes, with the
#   reason in the comment beside it, for coverage), which is the mechanism those
#   files already use.
#
# The re-run guard is the [views.ask-refactoring] view in .mindtape/config.toml:
# while any task the sweep filed is still open, the sweep refuses. One sprint of
# mechanical work is emptied before the next is filed.
#
# Adding a signal is a scoring function plus a template, not a config change:
# the `swept` / `size` / `mutants` / `coverage` tags are all declared already,
# and the guard selects on `swept` alone. Each signal is one file under
# `contrib/ask-refactoring/`, named in `ARMS` below and sourced from there;
# `signal_<name>` prints TSV rows on stdout and `emit_<name>` turns a row into a
# task. Nothing else may reach that stdout — a signal that shells out to a
# producer has to send its output to stderr, or it lands in the rows (see
# `ensure_lcov_fresh`).
#
# Not memoized with mmz, deliberately: the whole point is to re-observe the
# tree on every run. The one exception is `just cover` itself, inside the
# coverage signal: `just mmz cover` (see below) skips re-running the whole test
# suite when nothing under its scope changed, but the lcov it reads is still
# re-checked for freshness on every sweep — memoizing the expensive rebuild is
# not the same as memoizing the observation.
set -eo pipefail

repo="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo"

# ── the thresholds ──────────────────────────────────────────────────────────
#
# Every one of them is required and none is defaulted here, deliberately. The
# defaults live in the `ask-refactoring` recipe's `[arg(...)]` parameters and
# nowhere else (truth/tasks/derive-the-ask-refactoring-flag-table-from-arg-
# attributes.typ): that is what `just --usage ask-refactoring` renders, what
# `just --dump` carries into www/generated/sweep-flags.json, and what
# docs/contributing/signal-sweep.md's table is generated from. A fallback here
# would be a second copy of four numbers with nothing to keep it in step — the
# arrangement this whole change exists to end — and it would fail silently, at
# a threshold nobody chose, on the day the recipe stopped passing one.
#
# Initialised empty rather than left unset, because these five names are also
# the recipe's exported parameters: `$BASELINE` and friends are in this
# script's environment already, so an inherited value would satisfy the
# required-check below without the flag ever having been passed.
BASELINE=""
COVERAGE_FLOOR=""
CC_FLOOR=""
FN_COVERAGE_FLOOR=""
WRITE=""

usage() {
    cat <<'EOF'
usage: bash contrib/ask-refactoring.sh --write 0|1 --baseline PCT
           --coverage-floor PCT --cc-floor N --fn-coverage-floor PCT

Run it as `just ask-refactoring` — the recipe owns every default and forwards
all five flags; `just --usage ask-refactoring` prints them with their values.
All five are required here; there are no defaults at this layer.

env:
  MT   path to the `mt` binary (default: build and use ./target/debug/mt)
EOF
}

while [ $# -gt 0 ]; do
    case "$1" in
        --write) WRITE="$2"; shift 2 ;;
        --write=*) WRITE="${1#*=}"; shift ;;
        --baseline) BASELINE="$2"; shift 2 ;;
        --baseline=*) BASELINE="${1#*=}"; shift ;;
        --coverage-floor) COVERAGE_FLOOR="$2"; shift 2 ;;
        --coverage-floor=*) COVERAGE_FLOOR="${1#*=}"; shift ;;
        --cc-floor) CC_FLOOR="$2"; shift 2 ;;
        --cc-floor=*) CC_FLOOR="${1#*=}"; shift ;;
        --fn-coverage-floor) FN_COVERAGE_FLOOR="$2"; shift 2 ;;
        --fn-coverage-floor=*) FN_COVERAGE_FLOOR="${1#*=}"; shift ;;
        -h | --help) usage; exit 0 ;;
        *) echo "ask-refactoring: unknown argument: $1" >&2; usage >&2; exit 2 ;;
    esac
done

# Missing, or not a number: refuse rather than sweep at a threshold nobody
# chose. The recipe's patterns already reject a non-numeric value at dispatch,
# so this fires only for a direct `bash contrib/ask-refactoring.sh` call —
# which is exactly the caller the recipe's defaults cannot reach.
for var in WRITE BASELINE COVERAGE_FLOOR CC_FLOOR FN_COVERAGE_FLOOR; do
    flag="--$(printf '%s' "$var" | tr 'A-Z_' 'a-z-')"
    value="${!var}"
    if [ -z "$value" ]; then
        echo "ask-refactoring: missing $flag (no default at this layer)" >&2
        usage >&2
        exit 2
    fi
    case "$value" in
        *[!0-9]*)
            echo "ask-refactoring: $flag wants a number, got '$value'" >&2
            exit 2
            ;;
    esac
done

# Use the worktree's own `mt`, never whatever is on PATH: the sweep depends on
# `--dedup` and `--format json`, which a system-wide install may predate, and a
# tracker that files tasks with a stale binary is worse than one that does not
# run. Same convention as `just check-doc-coverage`.
MT="${MT:-}"
if [ -z "$MT" ]; then
    cargo build -p mindtape-cli -q
    MT="$repo/target/debug/mt"
fi

# ── the re-run guard ────────────────────────────────────────────────────────
open="$("$MT" ls '@ask-refactoring' --format path)"
if [ -n "$open" ]; then
    count="$(printf '%s\n' "$open" | grep -c .)"
    echo "ask-refactoring: $count task(s) from the last sweep are still open:" >&2
    printf '%s\n' "$open" | sed 's/^/  /' >&2
    cat >&2 <<'EOF'

Empty the view before sweeping again — finish them, cancel them, or write the
override that makes the finding stop existing. Filing a second sprint on top of
the first is how a signal-driven backlog becomes noise.

  mt ls @ask-refactoring
EOF
    exit 1
fi

# A stem derived from the finding, not from the title: stable across runs, so
# `--dedup skip` recognises the same finding twice, and namespaced by signal so
# the arms cannot collide. `$1` is the signal name (`size`, `coverage`,
# ...), `$2` the finding's path. Shared, so it stays here rather than in an arm.
#
# Operator characters are spelled out before the general substitution. Without
# it `replace > with >=` and `replace > with <` sanitize to the same run of
# hyphens, and `--dedup skip` then drops the second as "already filed" — the
# mutants arm folded thirteen stems over thirty-one survivors that way. Paths
# and function names carry none of these characters, so the size, coverage and
# undertested stems are unchanged.
stem_for() {
    local encoded
    encoded="$(printf '%s' "$2" \
        | sed -e 's/&&/and/g' -e 's/||/or/g' \
              -e 's/!=/ne/g' -e 's/==/eq/g' -e 's/>=/ge/g' -e 's/<=/le/g' \
              -e 's/->/to/g' -e 's/+=/plus-eq/g' -e 's/-=/minus-eq/g' -e 's/\*=/star-eq/g' \
              -e 's/>/gt/g' -e 's/</lt/g' -e 's/!/not/g' -e 's/+/plus/g' \
              -e 's/\*/star/g' -e 's/%/percent/g' -e 's/\^/caret/g' -e 's/~/tilde/g' \
              -e 's/=/eq/g' -e 's/&/and/g' -e 's/|/or/g')"
    printf '%s-%s\n' "$1" "$(printf '%s' "$encoded" | tr -c 'A-Za-z0-9' '-' | tr -s '-' | sed 's/^-//; s/-$//')"
}

# ── the arms ────────────────────────────────────────────────────────────────
#
# One file per signal, each defining `score_<arm>`, `signal_<arm>` and
# `emit_<arm>` plus the block of reasoning that justifies its numbers. Splitting
# on this seam is what keeps every file under the `Shell` cap in `.linecop.yaml`
# — the sweep files tasks about oversized files and was, until that cap existed,
# the largest source file in the repo and structurally unable to see itself.
#
# Adding an arm is: a new file here, its name in `ARMS`, and a template. Still
# no config change.
#
# `undertested` follows `coverage`: it defers to that arm (it stays quiet about a
# function whose file the file-shaped arm already reports) and borrows its
# freshness check and production-file predicate, so the pair reads in the order
# the deferral runs. Sourcing is a separate loop, so nothing here depends on the
# order for definitions — only the output does.
ARMS=(size coverage undertested mutants)

for arm in "${ARMS[@]}"; do
    # shellcheck source=/dev/null
    . "$repo/contrib/ask-refactoring/$arm.sh"
done

# Each signal's rows are captured with `$(...)` first, not read straight off a
# `< <(...)` process substitution: a signal that hits its own `exit 1` (a
# malformed linecop report, a stale lcov) is running in a subshell either way,
# but only the command-substitution form propagates that exit through `set -e`
# to this script. Piped directly into a `read` loop via process substitution,
# the same `exit 1` would kill only the substitution's subshell — the loop
# would just see end-of-input, "0 rows from this signal" would look identical
# to "this signal is refusing to run", and the sweep would report a clean,
# wrong result. Verified with a minimal repro before relying on it here.
findings=0
for arm in "${ARMS[@]}"; do
    rows="$("signal_$arm")"
    while IFS=$'\t' read -r -a row; do
        [ "${#row[@]}" -gt 0 ] || continue
        "emit_$arm" "${row[@]}"
        findings=$((findings + 1))
    done <<<"$rows"
done

if [ "$findings" -eq 0 ]; then
    echo "ask-refactoring: no findings at --baseline $BASELINE / --coverage-floor $COVERAGE_FLOOR" \
        "/ --cc-floor $CC_FLOOR --fn-coverage-floor $FN_COVERAGE_FLOOR"
elif [ "$WRITE" -eq 0 ]; then
    echo "ask-refactoring: $findings finding(s); re-run with --write to file them"
else
    echo "ask-refactoring: $findings finding(s) swept"
fi
