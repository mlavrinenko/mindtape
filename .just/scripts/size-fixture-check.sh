#!/usr/bin/env bash
# .just/scripts/size-fixture-check.sh — the body of the `size-fixture-check`
# recipe. Runs from the repo root, like the recipe that invokes it.
#
# Lock the file-size gate's scan scope, not its limits: synthesize an
# oversized shell script at `.just/scripts/` inside a replica tree, then
# assert `linecop`, reading this repo's `.linecop.yaml`, reports it. The
# limits themselves need no self-test — a cap either fires or the number is
# wrong, and either is visible. What is invisible, and what this gate exists
# for, is a walker that never reaches the file: linecop skipped dot-prefixed
# directories until `include_hidden: true`, so every recipe body the Justfile
# extraction moved into `.just/scripts/` was uncapped for as long as the
# extraction claimed the opposite (truth/tasks/restore-size-gate-coverage-
# over-the-dot-just-scripts.typ). A silently unscanned directory reports
# "All files within size limits." — the same output a clean tree gives.
#
# Why A replica rather than a probe in the real `.just/scripts/`: `just check`
# runs its arms in parallel, and the real `check-file-size` arm is one of
# them. A 401-line file appearing in the tracked tree mid-run would fail that
# arm, nondeterministically, for a reason no reader could place — and an
# interrupted run would leave the junk file behind. The replica carries the
# repo's own config file verbatim, so it still tests the real policy: a
# `.just` entry added to `exclude_dirs`, or `include_hidden` flipped back off,
# fails here.
#
# Both directions are asserted, for fmt-fixture-check.sh's reason. Arm one
# alone would pass on any linecop that reports the probe for some unrelated
# reason; arm two re-runs the identical scan against the same config with
# `include_hidden` deleted and requires the probe to go unreported, which is
# what pins the pass to the key this gate is about. Arm two is also the
# regression test for the bug itself — it reproduces the old behaviour on
# demand.
set -eo pipefail
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

probe_rel=".just/scripts/_probe.sh"
probe="$tmp/$probe_rel"
mkdir -p "$(dirname "$probe")"
# One line over the Shell cap read from the config, never a literal 401: the
# cap is .linecop.yaml's to set, and a hard-coded number here would turn a
# deliberate cap change into a failure of this gate instead of a pass.
cap=$(yq -r '.limits.Shell' .linecop.yaml)
{
    echo '#!/usr/bin/env bash'
    for _ in $(seq "$cap"); do echo 'echo probe'; done
} > "$probe"

# Assert the fixture exists and is genuinely over the cap before asserting the
# gate rejects it. A negative test whose fixture was never written passes for
# the wrong reason — the tool fails, or reports nothing, over an absent file
# and the assertion reads that as the bug being caught. This repo has shipped
# that once already, which is why the task asked for the existence check by
# name rather than leaving it to taste.
[ -f "$probe" ] || { echo "size-fixture-check: probe was never written to $probe" >&2; exit 1; }
lines=$(wc -l < "$probe")
if [ "$lines" -le "$cap" ]; then
    echo "size-fixture-check: probe is $lines lines, not over the $cap-line Shell cap — the fixture cannot bite" >&2
    exit 1
fi

cp .linecop.yaml "$tmp/.linecop.yaml"
if reported=$(linecop --config "$tmp/.linecop.yaml" --format paths "$tmp"); then
    echo "size-fixture-check: linecop reported nothing for a $lines-line file at $probe_rel" >&2
    echo "  the size gate does not reach .just/ — check include_hidden in .linecop.yaml" >&2
    exit 1
fi
if ! grep -qF "$probe_rel" <<<"$reported"; then
    echo "size-fixture-check: linecop failed but never named $probe_rel — it rejected something else" >&2
    echo "  reported: $reported" >&2
    exit 1
fi

# Arm two: the same scan, the same probe, `include_hidden` deleted. The probe
# must go unreported — proving the arm above passed because of that key.
yq -y 'del(.include_hidden)' .linecop.yaml > "$tmp/no-hidden.yaml"
if unreported=$(linecop --config "$tmp/no-hidden.yaml" --format paths "$tmp"); then
    [ -z "$unreported" ] || {
        echo "size-fixture-check: unexpected violation with include_hidden removed: $unreported" >&2
        exit 1
    }
else
    echo "size-fixture-check: linecop still reported the probe with include_hidden removed" >&2
    echo "  the arm above proves nothing about include_hidden — this gate is miswired" >&2
    exit 1
fi

# The size arm of `ask-refactoring` scores a test module as a test. Feed
# `score_size` one themed test module of a `mod.rs` and require the flavour
# column to read `test`, as it does for `tests.rs` and `*_tests.rs`.
# shellcheck source=/dev/null
source contrib/ask-refactoring/size.sh
for path in crates/x/src/config/tests_id.rs crates/x/src/config/tests.rs crates/x/src/config/id_tests.rs; do
    flavour=$(printf '[{"path":"%s","language":"Rust","lines":380,"limit":400,"baseline-limit":360}]' "$path" \
        | score_size | cut -f4)
    if [ "$flavour" != test ]; then
        echo "size-fixture-check: score_size read $path as '$flavour', not a test" >&2
        exit 1
    fi
done

# A rendered file joins its source's finding: one row, keyed by the source, that
# measures both files. `size_meta` reads the `Generated from` banner off the
# render and the `<mindtape-md>` block off the source.
report='[{"path":"./AGENTS.md","language":"Markdown","lines":75,"limit":78,"baseline-limit":70},
         {"path":"./docs/src/agents.typ","language":"Typst","lines":158,"limit":166,"baseline-limit":149}]'
rows=$(score_size "$(size_meta <<<"$report")" <<<"$report")
if [ "$(wc -l <<<"$rows")" -ne 1 ] || [ "$(cut -f1,2 <<<"$rows")" != $'docs/src/agents.typ\tprose' ]; then
    echo "size-fixture-check: AGENTS.md and its source did not fold into one prose finding:" >&2
    echo "$rows" >&2
    exit 1
fi
measured=$(cut -f3 <<<"$rows")
for path in AGENTS.md docs/src/agents.typ; do
    grep -qF "#raw(\"$path\")" <<<"$measured" || {
        echo "size-fixture-check: the folded finding does not measure $path: $measured" >&2
        exit 1
    }
done
