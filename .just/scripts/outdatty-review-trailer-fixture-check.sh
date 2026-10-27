#!/usr/bin/env bash
# .just/scripts/outdatty-review-trailer-fixture-check.sh — the body of the
# `outdatty::trailer-fixture` recipe
# (truth/tasks/require-a-docs-unaffected-trailer-when-an-outdatty-re-record-
# edits-no-dependent.typ). Runs from the repo root, like the recipe that
# invokes it.
#
# Lock .just/scripts/outdatty-review-trailer.sh, which `.githooks/commit-msg`
# runs, against the commits it must refuse and the ones it must let through.
# The check only runs inside a real `git commit`, and what it reads (the index
# git hands the hook, the message file, the amend and the merge in progress)
# exists only there, so each case is a real commit in a temp repo whose
# `core.hooksPath` runs the script. The lock is hand-written: the check reads
# the lock's diff and never hashes a file, so the fixture needs no outdatty.
set -eo pipefail

check="$PWD/.just/scripts/outdatty-review-trailer.sh"
[ -f "$check" ] || { echo "outdatty-review-trailer-fixture-check: $check is missing" >&2; exit 1; }

# The gate may run under a git hook (`just land`), whose variables would point
# the fixture's git at this repo. `BASH_ENV` goes too: a host hook there may
# rebuild `PATH` from the `.envrc` the temp repo lacks, losing yq.
unset GIT_DIR GIT_WORK_TREE GIT_INDEX_FILE GIT_OBJECT_DIRECTORY GIT_PREFIX BASH_ENV

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
repo="$tmp/repo"
mkdir -p "$repo/src" "$repo/docs" "$repo/.config" "$tmp/hooks"
# shellcheck disable=SC2016 # "$1" is the hook's own argument, expanded when it runs
printf '#!/usr/bin/env bash\nexec bash %q "$1"\n' "$check" >"$tmp/hooks/commit-msg"
chmod +x "$tmp/hooks/commit-msg"

g() { git -C "$repo" "$@"; }
g init -q -b main
g config user.name fixture
g config user.email fixture@example.invalid
g config commit.gpgsign false
g config core.hooksPath "$tmp/hooks"

fail() {
    echo "outdatty-review-trailer-fixture-check: $*" >&2
    exit 1
}

# The lock, where this repo keeps it: ls-docs and add-docs share
# docs/rules.typ, so a re-record of one carries the shared file's hash along
# in the other. src-only has no dependents. $1 ls-docs source, $2
# docs/rules.typ in both groups, $3 add-docs source, $4 src-only source, $5
# any extra group text.
lock() {
    cat >"$repo/.config/outdatty.lock" <<EOF
version: 1
algorithm: blake3
groups:
  add-docs:
    source:
      src/add.rs: $3
    dependents:
      docs/add.typ: a1
      docs/rules.typ: $2
  ls-docs:
    source:
      src/ls.rs: $1
    dependents:
      docs/ls.typ: l1
      docs/rules.typ: $2
  src-only:
    source:
      src/x.rs: ${4:-x1}
    dependents: {}
${5:-}
EOF
}

for f in src/ls.rs src/add.rs src/x.rs docs/ls.typ docs/add.typ docs/rules.typ; do
    echo one >"$repo/$f"
done
lock l1 r1 a1
g add -A
g commit -q -m "chore: base" >/dev/null 2>&1 || fail "the base commit, which has no lock at HEAD, was refused"
g tag base

reset() { g reset -q --hard base; }
edit() { local f; for f in "$@"; do echo two >>"$repo/$f"; done; }

# One commit of the staged tree with the message $2.., asserting exit $1 and
# that each needle shows in the output (`NOT:x` forbids x). The commit is
# undone after, so every case starts from `base`.
expect() { # <label> <want: 0|1> <message> <needle>...
    local label=$1 want=$2 msg=$3 rc=0 needle
    shift 3
    g add -A
    g commit -q -m "$msg" >"$tmp/out" 2>&1 || rc=$?
    if [ "$want" -eq 0 ]; then
        [ "$rc" -eq 0 ] || fail "$label: refused, want accepted: $(head -c 900 "$tmp/out")"
    else
        [ "$rc" -ne 0 ] || fail "$label: accepted, want refused"
    fi
    for needle in "$@"; do
        if [ "${needle#NOT:}" != "$needle" ]; then
            ! grep -qF -- "${needle#NOT:}" "$tmp/out" || fail "$label: named ${needle#NOT:}: $(head -c 900 "$tmp/out")"
        else
            grep -qF -- "$needle" "$tmp/out" || fail "$label: did not name $needle: $(head -c 900 "$tmp/out")"
        fi
    done
}

# A re-record with no dependent edit. The carried rules.typ hash is no review,
# and add-docs, whose source did not move, is not asked for one.
reset; edit src/ls.rs; lock l2 r2 a1
expect "re-record, no dependent edit" 1 "feat: ls" \
    "ls-docs" "docs/ls.typ" "docs/rules.typ" "Docs-Unaffected: ls-docs" "NOT:add-docs"

reset; edit src/ls.rs; lock l2 r2 a1
expect "re-record with the trailer" 0 "$(printf 'feat: ls\n\nBody.\n\nDocs-Unaffected: ls-docs\nRefs: x')"

reset; edit src/ls.rs; lock l2 r2 a1
expect "trailer key in lower case" 0 "$(printf 'feat: ls\n\ndocs-unaffected: ls-docs')"

reset; edit src/ls.rs; lock l2 r2 a1
expect "trailer naming another group" 1 "$(printf 'feat: ls\n\nDocs-Unaffected: add-docs')" "Docs-Unaffected: ls-docs"

reset; edit src/ls.rs; lock l2 r2 a1
expect "the name in the body, not a trailer" 1 "$(printf 'feat: ls\n\nDocs-Unaffected: ls-docs\nis what I would write.')"

reset; edit src/ls.rs docs/ls.typ; lock l2 r2 a1
expect "re-record with a dependent edit" 0 "feat: ls and its page"

reset; edit docs/rules.typ; lock l1 r2 a1
expect "a dependent recorded alone" 0 "docs: rules"

reset; edit src/x.rs; lock l1 r1 a1 x2
expect "a group with no dependents" 0 "feat: x"

reset; edit src/ls.rs; lock l1 r1 a1 x1 "$(printf '  new-docs:\n    source:\n      src/ls.rs: l2\n    dependents:\n      docs/ls.typ: l1')"
expect "a group new in this commit" 0 "dev: new group"

reset; edit src/ls.rs src/add.rs; lock l2 r2 a2
expect "two groups, one trailer" 1 "$(printf 'feat: both\n\nDocs-Unaffected: ls-docs')" \
    "Docs-Unaffected: add-docs" "docs/add.typ" "NOT:Docs-Unaffected: ls-docs"

reset; edit src/ls.rs src/add.rs; lock l2 r2 a2
expect "two groups, two trailers" 0 "$(printf 'feat: both\n\nDocs-Unaffected: ls-docs\nDocs-Unaffected: add-docs')"

reset; edit src/ls.rs
expect "a source edit with no lock change" 0 "feat: ls"

# An amend is judged against HEAD^, so dropping the trailer while rewording a
# re-record is refused, though the amend itself stages nothing.
reset; edit src/ls.rs; lock l2 r2 a1
expect "amend base" 0 "$(printf 'feat: ls\n\nDocs-Unaffected: ls-docs')"
rc=0
g commit -q --amend -m "feat: ls, reworded" >"$tmp/out" 2>&1 || rc=$?
[ "$rc" -ne 0 ] || fail "amend dropping the trailer: accepted, want refused"
grep -qF "Docs-Unaffected: ls-docs" "$tmp/out" || fail "amend: did not name the trailer: $(head -c 900 "$tmp/out")"

# A merge commit's diff against its first parent is the other branch's work,
# whose own commits carried the review.
reset; g checkout -q -b side
edit src/ls.rs; lock l2 r2 a1
expect "side re-record" 0 "$(printf 'feat: ls\n\nDocs-Unaffected: ls-docs')"
g checkout -q main; edit src/add.rs; g commit -qam "feat: add" >/dev/null 2>&1
g merge -q --no-ff -m "Merge branch 'side'" side >"$tmp/out" 2>&1 \
    || fail "merge of a reviewed branch: refused: $(head -c 900 "$tmp/out")"

echo "outdatty-review-trailer-fixture-check: ok"
