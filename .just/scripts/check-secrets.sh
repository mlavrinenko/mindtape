#!/usr/bin/env bash
# .just/scripts/check-secrets.sh — the body of the `check-secrets` recipe
# (truth/tasks/scan-the-tree-for-secrets-with-gitleaks.typ). Runs from the
# repo root, like the recipe that invokes it.
#
# WHAT IT SCANS: the working tree as it stands, under `.gitleaks.toml` —
# gitleaks' own default ruleset plus this repo's two allowlists, both argued
# in that file rather than restated here. History is out of scope; the reason
# is argued there too.
#
# WHY THE TARGET IS `.` AND THE CWD IS THE REPO ROOT, measured rather than
# assumed: gitleaks reports a finding's path exactly as its target was
# spelled, so `gitleaks dir "$PWD"` yields `/home/…/outdatty.lock` and every
# `^`-anchored allowlist in .gitleaks.toml silently stops matching — the
# gate would go from one known false positive to several, and the walk bound
# would stop bounding. A relative `.` from the root is the only invocation
# those anchors describe. It is also why arm one scans its fixture by cd-ing
# into it rather than by passing its path: a control that does not reproduce
# the real invocation pins the wrong thing.
#
# ARM ONE IS A CONTROL, because this gate's failure mode is a FALSE GREEN and
# nothing else. "no leaks found" is what a clean tree prints, what an
# over-broad allowlist prints, and what a config that failed to load its rules
# prints. So one synthesized fixture, scanned through the real config before
# the real tree is, has to report an exact set of findings — and that single
# assertion pins six separate things at once:
#
#   - the default ruleset is live at all (`aws-access-token` fires),
#   - the path allowlist fires and is `^`-anchored (the copy under `target/`
#     is silent),
#   - the outdatty.lock allowlist fires (the digest line is silent),
#   - it is scoped to the DIGEST SHAPE (a `generic-api-key` hit in that same
#     file that is not a bare sha256 is still reported — dropping
#     `condition = "AND"` from the config fails here and nowhere else),
#   - it is scoped to the RULE (another rule's finding in that file survives),
#   - and no allowlist is broad enough to swallow the tree.
#
# It runs in a temp dir rather than in the tracked tree for size-fixture-
# check.sh's reason: `just check` runs its arms in parallel, and a real
# credential shape written under the repo would be read as real by arm two of
# a concurrent run.
#
# ARM THREE IS ABOUT THE MANIFEST, not about secrets, and it is here rather
# than in script-scope-check.sh because its subject is THIS rule's own input
# list. Arm two reads the whole tree, and .mmz/config.yaml cannot say so — it
# has no negative glob, and the one directory a gate-tagged rule may not name
# is `truth/tasks/**` (the NOTE ON SCOPE there: `mt flip` writes the task file
# it is gating, so the close gate would flip itself stale mid-command). What
# is left is an enumeration, and an enumeration goes false-fresh the moment
# a top-level directory arrives that nobody listed — a whole subtree that
# could gain a credential without ever busting the record. So the enumeration
# is checked instead of trusted, against mmz's OWN expansion rather than a
# second implementation of its globs.
#
# WHAT ARM THREE DELIBERATELY DOES NOT DEMAND: `truth/tasks/**`. Those files
# ARE scanned by arm two and are not memoized, exactly the trade `just
# fmt-check` already takes over the same directory — a full `just check`
# still refuses a credential in a task file; a task-file-only edit just does
# not bust this rule's cached pass.
#
# ARM FOUR IS ABOUT THE WALK BOUND rather than the tree, and lives in
# .just/scripts/_gitleaks-bound-check.sh rather than here — a separate subject
# (the manifest, not secrets) that arm two's verdict never depends on, and the
# 60 lines that made this file 96% of the 300-line Shell cap. Argued there.
set -eo pipefail

config=".gitleaks.toml"
rule="just check-secrets"

if [ ! -f "$config" ]; then
    echo "check-secrets: $config is missing — running outside the repo root?" >&2
    exit 1
fi

# arm one

# Both literals carry `gitleaks:allow`, or arm two would report THIS file: it
# is a tracked file in the tree arm two scans, and the fixture below has to
# hold the bare credential shape for the control to mean anything. The
# comment is on the assignment only — what gets written to the fixture is the
# value, so the copy under the scanner is unannotated.
control_key="AKIAZZ4XK3PLQ2NR7WVB"                                              # gitleaks:allow
control_digest="0f3a9c1d7e2b48605af9c3d21e8b7460f5a2c9d31e4b8760a3f9c2d51e8b7463" # gitleaks:allow
control_nondigest="Zx91Qw82Mn73Bv64Ct55Dp46Er37Fg28Hi19Jk"                      # gitleaks:allow

repo="$PWD"
fixture=$(mktemp -d)
trap 'rm -rf "$fixture"' EXIT

# Resolved to an absolute path while `$PWD` is still the repo root: a shell
# with a direnv PATH hook (this repo's dev shell) rebuilds PATH off `$PWD` on
# every command, so a bare `gitleaks` looked up again after the `cd` below —
# arm one's control has to run from inside the fixture, not the repo root,
# for the same reason it cd's rather than passes a path (see above) — would
# silently fall back to whatever (if anything) sits outside the dev shell.
gitleaks_bin=$(command -v gitleaks) || {
    echo "check-secrets: gitleaks not found on PATH" >&2
    exit 1
}

mkdir -p "$fixture/target"
# Three lines, one per thing the outdatty.lock allowlist must get right. The
# first is a synthetic stand-in for the real lock's `refactor-auth.typ` entry
# — keyed on a name carrying "auth" so `generic-api-key` reads it as a secret
# — and must be silenced. Synthetic rather than copied, so re-recording that
# group cannot rot this fixture. The second is the SAME rule in the SAME file
# on a value that is not a bare sha256, and must survive: it is the only line
# that fails if the allowlist is widened from the digest shape to the whole
# file. The third is a different rule entirely, and must survive too.
cat >"$fixture/outdatty.lock" <<EOF
  refactor-auth.typ: $control_digest
  refactor-auth-key.typ: $control_nondigest
  leaked.typ: $control_key
EOF
printf 'key = %s\n' "$control_key" >"$fixture/target/leak.txt"

# `--exit-code 0` so a finding is not an error here — the finding IS the
# expected result, and the assertion is on the report rather than on the exit
# status. A gitleaks that fails for any other reason (an unparseable config,
# most likely) writes no report, which is caught separately below rather than
# left to `set -e` to end the script with no explanation.
report="$fixture/control.json"
if ! (cd "$fixture" && "$gitleaks_bin" dir -c "$repo/$config" --no-banner --exit-code 0 \
    --report-format json --report-path "$report" . >"$fixture/control.log" 2>&1); then
    echo "check-secrets: gitleaks could not scan the control fixture" >&2
    sed 's/^/  /' "$fixture/control.log" >&2
    exit 1
fi

expected="outdatty.lock:aws-access-token, outdatty.lock:generic-api-key"
control=$(jq -r '[.[] | "\(.File):\(.RuleID)"] | sort | join(", ")' "$report")
if [ "$control" != "$expected" ]; then
    echo "check-secrets: the control fixture reported [$control]" >&2
    echo "  expected exactly [$expected]" >&2
    echo "  the scan below cannot be trusted — fix $config, not the fixture" >&2
    exit 1
fi

# arm two

# THE SCAN HOLDS target/www-generated.lock, because the walk bound in $config
# keeps this arm out of that directory's CONTENTS and cannot keep it out of
# the stat of the entry itself — the strace measurement, and why no pattern
# can buy that, are argued beside the bound in $config. www/generate.sh
# `rm -rf`s the directory on entry and `just check` is [parallel], so without
# the lock a deletion landing in that window logs an `ERR` on a run that
# finds no leaks and exits 0 (truth/tasks/check-secrets-still-logs-an-err-
# over-www-generated.typ). Same fd-9 flock www/generate.sh,
# docs/generate-md.sh and check-doc-facts.sh share — this arm is one more
# READER of that directory, and was the last one not taking it. No
# MINDTAPE_GENERATED_LOCK handshake: nothing holding the lock calls this
# gate, and an unconditional acquisition makes a future caller deadlock
# loudly rather than race quietly.
mkdir -p target
exec 9>target/www-generated.lock
flock 9

# Asserted rather than assumed — deleting the three lines above restores the
# race in silence, on a gate that stays green either way. `flock -n` opens
# the file on a NEW description, which conflicts with the one fd 9 holds, so
# it can only succeed if the lock is not ours.
if flock -n target/www-generated.lock true 2>/dev/null; then
    echo "check-secrets: the tree scan is not holding target/www-generated.lock" >&2
    echo "  restore the \`exec 9>…; flock 9\` above — without it a parallel docs" >&2
    echo "  arm deletes www/generated mid-walk and gitleaks ERRs on a green run" >&2
    exit 1
fi

# The report is read rather than the console output, and not for tidiness:
# gitleaks 8.30 prints `leaks found: N` and nothing else unless `-v` is
# passed, so a failing gate would name no file at all. Reading the JSON gives
# the file and line a reader has to act on — and gives them WITHOUT the
# secret, which matters because this arm's output is what a failing CI job
# publishes. `--redact` covers gitleaks' own logging for the same reason.
#
# `--exit-code 0` for arm one's reason: a finding is not an error to be
# trapped here, it is the thing being measured, and it has to be
# distinguishable from gitleaks failing to run at all.
scan="$fixture/scan.json"
if ! gitleaks dir -c "$config" --no-banner --redact --exit-code 0 \
    --report-format json --report-path "$scan" .; then
    echo "check-secrets: gitleaks could not scan the working tree" >&2
    exit 1
fi

findings=$(jq -r '.[] | "  \(.File):\(.StartLine)  \(.RuleID)"' "$scan" | sort)
if [ -n "$findings" ]; then
    echo "check-secrets: gitleaks reported $(wc -l <<<"$findings") secret(s) in the working tree" >&2
    printf '%s\n' "$findings" >&2
    echo "  a real one is sops-encrypted or removed, never allowlisted" >&2
    echo "  a false positive is allowlisted in $config, scoped to the rule that" >&2
    echo "  misfired and to what it matched — never by fingerprint, which expires" >&2
    echo "  re-run \`gitleaks dir -c $config -v .\` to see what matched" >&2
    exit 1
fi

# Released the moment the walk is over: every [parallel] arm that touches
# www/generated/ is blocked while it is held, and neither arm below reads
# that directory — mmz filters its own walk by www/.gitignore.
exec 9>&-

# -------------------------------------------------------------- arm three

# mmz's own expansion of the rule's declared inputs, so this arm compares
# against what mmz actually hashes rather than against a second reading of
# the globs. An empty list means the rule is gone or renamed, which is a
# failure of this arm rather than a pass with nothing to compare.
hashed=$(mmz --status=json | jq -r --arg r "$rule" '
    .rules[] | select(.name == $r) | .inputs[].path
' | sort)
if [ -z "$hashed" ]; then
    echo "check-secrets: .mmz/config.yaml declares no \`$rule\` rule" >&2
    echo "  the rule this arm reads was renamed — retarget the arm, do not delete it" >&2
    exit 1
fi

# Tracked files only: an untracked one cannot carry a COMMITTED credential,
# which is the whole subject. Symlinks are dropped (mode 120000) because
# neither side reads them — gitleaks skips a symlink without
# `--follow-symlinks`, and mmz hashes no link either, so CLAUDE.md would
# otherwise be reported as an uncovered file that nothing scans.
scanned=$(git ls-files -s | grep -v '^120000 ' | cut -f2- | grep -v '^truth/' | sort)

uncovered=$(comm -23 <(printf '%s\n' "$scanned") <(printf '%s\n' "$hashed"))
if [ -n "$uncovered" ]; then
    n=$(wc -l <<<"$uncovered")
    echo "check-secrets: $n scanned files are outside \`$rule\`'s mmz inputs" >&2
    head -5 <<<"$uncovered" | while IFS= read -r path; do
        echo "  $path" >&2
    done
    [ "$n" -gt 5 ] && echo "  … and $((n - 5)) more" >&2
    echo "  add their top-level path to the \`secrets-tree\` scope in .mmz/config.yaml," >&2
    echo "  or this gate serves a stale pass over a subtree it never hashed" >&2
    exit 1
fi

# --------------------------------------------------------------- arm four

# Its control first, for the reason arm one runs one: a manifest check whose
# pass looks identical to a check that stopped checking. Then both gitignores
# the bound is a hand copy of, passed rather than discovered — the script
# derives each one's prefix from its own path, so a third is covered by being
# named here. Run rather than `exec`-ed, so the summary below stays this
# file's last word.
bash .just/scripts/_gitleaks-bound-control.sh
bash .just/scripts/_gitleaks-bound-check.sh "$config" .gitignore www/.gitignore

echo "check-secrets: no secrets in the tree, all $(wc -l <<<"$scanned") scanned files are in \`$rule\`'s inputs"
