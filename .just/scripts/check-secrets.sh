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
# ARM FOUR IS ABOUT THE WALK BOUND rather than the tree, and runs last
# because arm two's verdict never depends on it. Argued where it fires.
set -eo pipefail

config=".gitleaks.toml"
rule="just check-secrets"

if [ ! -f "$config" ]; then
    echo "check-secrets: $config is missing — running outside the repo root?" >&2
    exit 1
fi

# ---------------------------------------------------------------- arm one

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
if ! (cd "$fixture" && gitleaks dir -c "$repo/$config" --no-banner --exit-code 0 \
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

# ---------------------------------------------------------------- arm two

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

# THE WALK BOUND IS A HAND COPY, and this arm stops it drifting. gitleaks
# reads no .gitignore, so the unscoped `paths` allowlist in $config restates
# by hand what is generated scratch — and half of that list is owned by docs
# arms `just check` runs in PARALLEL with this one: www/generate.sh deletes
# and recreates www/generated/ on every run, `tola build` rewrites www/.tola/
# in place. Missing one costs no coverage (nothing there is committable); it
# makes arm two walk a directory that can vanish underneath it, which gitleaks
# logs at ERR while still exiting 0 — measured on 8.30, `skipping directory:
# could not get info error="lstat www/generated: no such file or directory"`.
# ERR on a green run teaches the reader to re-run rather than to read, which
# is the property that makes a real leak look like the same flake
# (truth/tasks/check-secrets-races-the-docs-arm-over-www-generated.typ).
#
# www/.gitignore IS THE DECLARATION both readers of this tree consult. mmz
# filters its input walk by it — which is why `secrets-tree`'s `www/**`
# resolves three files under www/assets/ and none of www/generated/, and why
# the mmz half of the same race was closed by a line there, not here. gitleaks
# cannot read it, so this arm demands the bound cover every path it names.
#
# WHY ONLY www/.gitignore. Every root-anchored entry there is docs-build
# output, no exception and no negation, so one rule covers the file. The root
# .gitignore is a mixed list — `/.cargo/*` carries a tracked exception
# (`!/.cargo/mutants.toml`) that bounding would hide from the scan, `/result`
# is a nix symlink, `/skills/*-workspace` is a glob — so the same rule there
# needs a per-entry decision, not a check. Filed, not pretended away.
#
# TWO SHAPES ARE NOT DEMANDED. An unanchored pattern (`.DS_Store`,
# `.goutputstream-*`) names a class of transient with no fixed location, and a
# path regex has nothing to say about it; the readers that must skip those do
# it through www/.gitignore directly. And the BARE path is what must match,
# never a child: `^www/public/` skips every entry inside www/public/ while
# still stat-ing www/public/ and descending into it, which bounds nothing —
# `^www/public(/|$)` does.
#
# The allowlist is selected by carrying no `targetRules`: a rule-scoped one is
# a false-positive silencer (arm one's subject), the unscoped one is the
# bound. tomlq is the `yq` package's TOML front end, the same dev-shell pin.
bound=$(tomlq -r '.allowlists[] | select(has("targetRules") | not) | .paths[]' "$config")
if [ -z "$bound" ]; then
    echo "check-secrets: $config declares no unscoped path allowlist" >&2
    echo "  the walk bound this arm reads is gone — restore it, do not delete the arm" >&2
    exit 1
fi

unbounded=""
while IFS= read -r entry; do
    # Root-anchored only — which also drops comments and blank lines.
    [ "${entry#/}" != "$entry" ] || continue
    path="www${entry%/}"
    grep -qE -f <(printf '%s\n' "$bound") <<<"$path" || unbounded="$unbounded  $path"$'\n'
done <www/.gitignore

if [ -n "$unbounded" ]; then
    echo "check-secrets: www/.gitignore names paths the walk bound in $config misses" >&2
    printf '%s' "$unbounded" >&2
    echo "  each is written by a docs arm \`just check\` runs in parallel with this scan," >&2
    echo "  so the walk must never enter it — add \`^<path>(/|\$)\` (a directory) or" >&2
    echo "  \`^<path>\$\` (a file) to the allowlist that declares no targetRules" >&2
    exit 1
fi

echo "check-secrets: no secrets in the tree, all $(wc -l <<<"$scanned") scanned files are in \`$rule\`'s inputs, and the walk bound covers www/.gitignore"
