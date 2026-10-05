#!/usr/bin/env bash
# .just/scripts/check-secrets-manifest.sh — arm three of
# .just/scripts/check-secrets.sh, split out under the Shell cap. Called by
# check-secrets.sh after its tree walk; takes no lock and reads nothing that
# walk holds.
#
# Arm three is about the manifest, not about secrets, and it lives beside the
# gate rather than in script-scope-check.sh because its subject is this rule's
# own input list. The tree scan in check-secrets.sh reads the whole tracked
# tree, and .mmz/config.yaml cannot say so — it has no negative glob, and the
# one directory a gate-tagged rule may not name is `truth/tasks/**` (the note
# on scope there: `mt flip` writes the task file it is gating, so the close
# gate would flip itself stale mid-command). What is left is an enumeration,
# and an enumeration goes false-fresh the moment a top-level directory arrives
# that nobody listed — a whole subtree that could gain a credential without
# ever busting the record. So the enumeration is checked instead of trusted,
# against mmz's own expansion rather than a second implementation of its
# globs.
#
# What it deliberately does not demand: `truth/tasks/**`. Those files are
# scanned by the tree arm and are not memoized, exactly the trade `just
# fmt-check` already takes over the same directory — a full `just check` still
# refuses a credential in a task file; a task-file-only edit just does not
# bust this rule's cached pass.
set -eo pipefail

rule="just check-secrets"

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

# Tracked files only: an untracked one cannot carry a committed credential,
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
