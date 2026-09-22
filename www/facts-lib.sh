# www/facts-lib.sh — the flock re-exec guard and the atomic emit helpers
# shared by www/generated/'s writers.
#
# Extracted verbatim out of www/generate-facts.sh
# (truth/tasks/extract-generate-facts-shs-flock-emit-preamble-into-facts-lib-sh.typ):
# that script sat at 298 of its 300-line linecop cap with no room for its next
# artifact. `source`d, not executed — a caller sets $SCRIPT_DIR/$REPO/$OUT
# first, since the guard and both helpers read them.
#
# Same lock as www/generate.sh, same reason: `just check` is [parallel] and
# check-doc-facts, docs::md-check and docs::check can all be inside this
# directory at once. Re-exec under flock unless a caller already holds it —
# www/generate.sh exports the variable before calling this script, so the
# wipe-then-restore there stays ONE critical section instead of two.
if [ -z "$MINDTAPE_GENERATED_LOCK" ]; then
  export MINDTAPE_GENERATED_LOCK=1
  exec flock "$REPO/target/www-generated.lock" "$0" "$@"
fi

# Every artifact below is written through a temp file and moved into place, so
# a reader holding no lock (a Typst source mid-render) sees the previous
# complete file or the next one, never a truncated or absent one. `mv` within
# one filesystem is atomic; a redirect straight onto the destination is not,
# and would also leave a zero-byte file behind when the producing jq program
# calls halt_error (the gate-group cross-check below does).
#
# The parse check makes that hold under a FAILING producer, not just a slow
# one: a jq that dies mid-stream still closes the pipe, so the temp file exists
# and is partial — and a plausible-looking JSON prefix reaches the reader as a
# fact, not as an error. Every artifact here is JSON, so `jq -e .` is the test.
emit() {
  local name="$1" tmp="$OUT/.$1.tmp.$$"
  cat >"$tmp"
  if ! jq -e . "$tmp" >/dev/null 2>&1; then
    rm -f "$tmp"
    echo "generate-facts.sh: refusing to install malformed $name" >&2
    return 1
  fi
  mv -f "$tmp" "$OUT/$name"
}

# Same discipline as emit() above, for an artifact that is not JSON — `jq -e .`
# has nothing to parse. just.typ's own producer below builds the whole file as
# ONE jq string value before printing anything, so a halted run writes nothing
# rather than a truncated prefix; refusing empty content is the raw equivalent
# of emit()'s parse check.
emit_raw() {
  local name="$1" tmp="$OUT/.$1.tmp.$$"
  cat >"$tmp"
  if [ ! -s "$tmp" ]; then
    rm -f "$tmp"
    echo "generate-facts.sh: refusing to install empty $name" >&2
    return 1
  fi
  mv -f "$tmp" "$OUT/$name"
}
