#!/usr/bin/env bash
# .just/scripts/script-attr-check.sh — the body of the `script-attr-check`
# recipe (truth/tasks/lock-the-script-attribute-against-a-shebang-
# regression.typ). Runs from the repo root, like the recipe that invokes it.
#
# `set script-interpreter := ["bash", "-eo", "pipefail"]` hands every `[script]`
# recipe strict mode by construction, which is what stopped `count-tests`
# printing a partial test count and exiting 0. The setting removed that mistake
# from the five recipes that existed; it removed nothing from the Justfile. A
# `#!/usr/bin/env bash` body still parses, still runs, and still runs outside
# strict mode, so the next multi-line recipe can be written straight back into
# the shape the conversion just left. This is the static self-test that stops
# it — the fourth on the shelf beside positional-args-check.sh,
# arg-pattern-check.sh and size-fixture-check.sh, and there for their reason: a
# convention nothing checks is a comment.
#
# TWO ARMS, because the attribute and the setting are each useless alone. The
# attribute routes a body to the interpreter; the setting decides what that
# interpreter is. Miss either and a body runs unstrict — arm 1 catches the body
# that never reaches the setting, arm 2 catches the setting that would not bite
# if it did.
#
# `.shebang` IS NOT THE DISCRIMINATOR for arm 1, measured on just 1.46 rather
# than assumed: a `[script]` recipe dumps `shebang: true` exactly as a `#!` body
# does, because the field means "this body is a script", not "this body opens
# with `#!`" — the same measurement www/gates.jq records where it reads the
# field. So arm 1 pairs the two: `shebang: true` AND no `script` key in
# `.attributes` is a bare `#!` body, and nothing else is. Attributes come back
# in two shapes, so the test accepts both rather than assuming one — a
# no-argument attribute serializes as a bare string (`"positional-arguments"`),
# an argument-taking one as an object (`{"script": null}`, null because
# `[script]` takes an OPTIONAL command).
#
# `[script("python3")]` passes arm 1 and is meant to. It names its own
# interpreter at the recipe, which is a visible choice rather than the silent
# escape this gate exists for, and demanding bash of a python body would be
# wrong. What it does not do is escape arm 2 — the file it lives in still has
# to carry a working setting for every other body in it.
#
# WHY ARM 2 RUNS A PROBE INSTEAD OF MATCHING THE SETTING'S TEXT. The dump does
# not carry the setting at all: `settings` in `just --dump --dump-format json`
# has no `script_interpreter` key on just 1.46 (measured — `shell` is there,
# this one is not), so the only static read available is a grep of the file,
# and a grep can only assert a spelling. Asserting the spelling is the wrong
# assertion anyway, in both directions, which the same measurement run shows:
# `["bash", "-euo", "pipefail"]` is strictly better than what is here today and
# fails a literal match, while `["bash", "-e"]` — pipefail dropped, which is the
# exact half that let `count-tests` exit 0 on a partial count — passes any match
# loose enough to admit the first. So the replica carries the file's own setting
# line verbatim, and the assertion is that a `false | true` probe DIES under it.
# That is the property. The spelling is not.
#
# WHY EVERY JUSTFILE, not only the ones with `[script]` recipes today. A module
# does not inherit the root's setting — measured on just 1.46: the identical
# `[script]` recipe fails under the root Justfile and prints its after-pipe
# marker with exit 0 in a module that omits the line. `.just/docs.just`
# therefore carries a setting no recipe there uses yet, which reads exactly like
# dead config to anyone tidying up, and deleting it would hand that module's
# first `[script]` recipe just's default `sh -eu` silently. Requiring the
# setting per FILE rather than per script recipe is what makes that line
# load-bearing today instead of on the day it is first needed.
#
# Nothing here runs a real recipe body. Arm 1 reads the dump; arm 2 runs only
# the synthesized probe in a temp dir — the replica discipline
# size-fixture-check.sh uses, for its reason: `just check` runs its arms in
# parallel, so a gate that executed anything in the tracked tree would race the
# arms checking it.
set -eo pipefail

fail=0
dump="$(just --dump --dump-format json)"
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

# Printed by the probe only if execution survived a failed pipeline. `set quiet`
# in the replica is what keeps this honest: without it just echoes the body, and
# the echoed `echo REACHED-AFTER-PIPE` line would match the grep on every run.
marker=REACHED-AFTER-PIPE

# ARM 1. Every recipe just calls a script must carry `[script]`, so the
# interpreter setting reaches it. `files` walks `.modules` recursively — the
# same walk gates.jq already needed for the gate table, and the reason is the
# same: a module is where the next long body is most likely to land, and one
# not walked here is one the gate silently exempts.
offenders=$(jq -r '
  def files:
    [{source: .source, recipes: (.recipes // {})}]
    + ((.modules // {}) | to_entries | map(.value) | map(files) | add // []);
  def carries_script:
    (.attributes // [])
    | any(. == "script" or (type == "object" and has("script")));
  files[]
  | .source as $src
  | (.recipes | to_entries[] | .value)
  | select(.shebang == true and (carries_script | not))
  | "\(.namepath)\t\($src)"
' <<<"$dump")

if [ -n "$offenders" ]; then
    while IFS=$'\t' read -r recipe src; do
        echo "script-attr-check: recipe '$recipe' (${src#"$PWD/"}) opens its body with a #! shebang" >&2
        echo "  drop the shebang line and add [script], so 'set script-interpreter' applies to it" >&2
        fail=1
    done <<<"$offenders"
else
    echo "script-attr-check: no recipe body opens with a #! shebang"
fi

# write_probe <replica-path> <setting-line>
# A standalone justfile carrying the setting under test and one `[script]`
# recipe whose first line fails only as a PIPELINE — `false | true` succeeds
# without pipefail and fails with it, so the probe separates that flag from `-e`
# rather than conflating the two.
write_probe() {
    printf 'set unstable := true\nset quiet := true\n%s\n\n[script]\nprobe:\n    false | true\n    echo %s\n' \
        "$2" "$marker" > "$1"
}

# The control, run once before the per-file loop. Arm 2 passes a file when the
# probe DIES, and a probe that dies for an unrelated reason (a broken replica, a
# just that cannot parse it, no bash) would read as every file passing. So first
# require the identical probe to SURVIVE a flagless bash: that pins each failure
# below to the flags in the setting line, which is the only thing that differs.
write_probe "$tmp/control.just" 'set script-interpreter := ["bash"]'
if control_out=$(just --justfile "$tmp/control.just" probe </dev/null 2>&1) \
    && grep -qF "$marker" <<<"$control_out"; then
    echo "script-attr-check: the probe survives a flagless bash, so a death below means the flags"
else
    echo "script-attr-check: the probe did not survive a flagless bash — it proves nothing about strict mode" >&2
    echo "  output: $control_out" >&2
    exit 1
fi

# ARM 2. Every justfile in the dump, root and modules alike.
sources=$(jq -r '
  def files: [.source] + ((.modules // {}) | to_entries | map(.value) | map(files) | add // []);
  files[]
' <<<"$dump")

n=0
while IFS= read -r src; do
    rel=${src#"$PWD/"}
    line=$(grep -E '^[[:space:]]*set[[:space:]]+script-interpreter[[:space:]]*:=' "$src" || true)
    if [ -z "$line" ]; then
        echo "script-attr-check: $rel declares no 'set script-interpreter'" >&2
        echo "  a justfile without one hands its [script] recipes just's default sh -eu; the setting does not cross a module boundary" >&2
        fail=1
        continue
    fi
    n=$((n + 1))
    write_probe "$tmp/probe-$n.just" "$line"
    if out=$(just --justfile "$tmp/probe-$n.just" probe </dev/null 2>&1); then
        echo "script-attr-check: $rel's script-interpreter ran 'false | true' to exit 0 — pipefail is not set" >&2
        fail=1
        continue
    fi
    if grep -qF "$marker" <<<"$out"; then
        echo "script-attr-check: $rel's script-interpreter continued past a failed pipeline — -e is not set" >&2
        fail=1
        continue
    fi
    echo "script-attr-check: $rel aborts a failed pipeline (${line#"${line%%[![:space:]]*}"})"
done <<<"$sources"

if [ "$fail" -ne 0 ]; then
    echo "script-attr-check: strict mode is not arriving by construction" >&2
    exit 1
fi
echo "script-attr-check: every script body goes through [script], and all $n justfile interpreters abort a failed pipeline"
