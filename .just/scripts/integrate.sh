#!/usr/bin/env bash
# .just/scripts/integrate.sh — the body of the `integrate` recipe
# (truth/tasks/assert-a-branchs-gate-before-fast-forwarding-it.typ). Runs from
# the repo root, like fork.sh, whose inverse it is.
#
# WHAT THIS IS FOR. `just fork` stands a delegated worktree up; this takes it
# back down, and the one thing it does on the way is an assertion nothing else
# makes. A FAST-FORWARD WRITES NO COMMIT, so `.githooks/pre-commit` — the only
# place `mmz --is-fresh --tag gate` fires on its own — never runs on the merge.
# The branch's own commits ran it, except the ones that carried a `Gate-Scope:`
# trailer and deliberately skipped it; that scoping is legitimate precisely
# because the full aggregate is supposed to run at integration. Without this
# recipe "at integration" means whatever the integrator typed, and what was
# typed was a pipeline ending in `tail` — which reports `tail`'s exit code, so
# a branch whose test arm failed read as green and landed on `main`.
#
# THE ASSERTION IS READ FROM THE WORKTREE, not from this checkout, and that is
# the design rather than a detail. mmz's cache records are per-tree, so "did
# the gate pass against the code being merged" can only be answered where that
# code is. It has to be asked before `main` moves and before the worktree is
# gone, which is why removal is the last line of this script.
#
# IT NEVER RUNS THE GATE, the same trade the pre-commit hook takes. Asserting a
# prior pass is cheap and is the predicate worth having; running the aggregate
# at merge time is slow at the least convenient moment, and a slow command at
# that moment is how a bypass gets invented.
#
# THERE IS NO WAIVER FLAG, unlike fork.sh's MINDTAPE_FORK_DIRTY_OK. One here
# would be a supported way to land a red branch, which is the exact event this
# recipe exists to make impossible to do quietly. The escape hatch is
# `git merge --ff-only <branch>` by hand: it is what exists today, it takes a
# deliberate second command, and nothing about it reads as blessed.
#
# THE BRANCH REF SURVIVES the worktree that held it, deliberately. Removing it
# is a separate judgement — a branch is cheap, and `git branch -d` after a
# successful fast-forward would delete the only name left pointing at the work
# if the integration turns out to have been the wrong call.
set -eo pipefail

branch="$1"

# `git worktree list --porcelain` is the only mapping from a branch to the
# directory holding it. A branch checked out nowhere has no tree to ask, which
# is also the shape of "the agent's worktree was already removed" — and a gate
# record that cannot be read is not a gate record that passed.
wt=$(git worktree list --porcelain | awk -v ref="refs/heads/$branch" '
    /^worktree / { path = substr($0, 10) }
    /^branch /   { if (substr($0, 8) == ref) { print path; exit } }
')
if [ -z "$wt" ]; then
    echo "integrate: no worktree has '$branch' checked out" >&2
    echo "  a branch with no worktree carries no gate record to read; \`just fork $branch\` stands one up" >&2
    exit 1
fi

here=$(git rev-parse --show-toplevel)
# Resolved once, up front, and tolerating a detached HEAD. Read lazily it would
# be read again on the LAST line — after the fast-forward has already landed —
# where `symbolic-ref` failing would report a successful integration as a failed
# one, which is the false red to match this recipe's false green.
onto=$(git symbolic-ref --short HEAD 2>/dev/null || echo "detached HEAD")
if [ "$wt" = "$here" ]; then
    echo "integrate: this checkout IS the worktree holding '$branch' — nothing here to fast-forward onto it" >&2
    echo "  run it from the checkout holding the branch you are merging INTO" >&2
    exit 1
fi

# Asked up front, though `git worktree remove` refuses a dirty tree anyway: it
# refuses at the END, after the fast-forward has already moved this checkout,
# and a half-done integration is worse than a refused one. The same reason
# fork.sh counts untracked files — a brand-new source file is the change with
# no committed version to fall back to, and removing the tree is where it dies.
dirty=$(git -C "$wt" status --porcelain)
if [ -n "$dirty" ]; then
    echo "integrate: $wt has uncommitted changes:" >&2
    sed 's/^/  /' <<<"$dirty" >&2
    echo "  commit them there (which re-runs the gate), or discard them, then integrate again" >&2
    exit 1
fi

# Checked before the gate rather than left to `git merge --ff-only`'s own
# refusal, so a branch that cannot land never spends a gate assertion arguing
# about it. Rebasing is out of scope on purpose: a conflict is a person's job.
branch_sha=$(git rev-parse --verify "refs/heads/$branch")
head_sha=$(git rev-parse --verify HEAD)
if ! git merge-base --is-ancestor "$head_sha" "$branch_sha"; then
    echo "integrate: '$branch' is not a fast-forward of $onto" >&2
    echo "  HEAD $head_sha  $branch $branch_sha — rebase the branch onto this checkout first" >&2
    exit 1
fi

# `cd` rather than `direnv exec "$wt"` alone: direnv exec loads that
# directory's environment but does NOT change the working directory (verified
# — `cd /tmp && direnv exec <repo> pwd` prints /tmp), and mmz resolves its
# manifest from cwd upward. Without the cd it would read THIS checkout's cache
# and answer about the wrong tree — silently, and green, which is the failure
# mode the whole recipe is against.
#
# The direnv hop itself is what puts mmz and every gate tool on PATH; fork.sh
# `direnv allow`s the worktree it creates, so it is there to be loaded. Falling
# back to a bare command is right rather than fatal for a worktree made some
# other way: a host with the tools already on PATH answers the same question.
run_in_worktree() {
    if command -v direnv >/dev/null 2>&1; then
        (cd "$wt" && direnv exec . "$@")
    else
        (cd "$wt" && "$@")
    fi
}

# The verdict is `--is-fresh`'s own exit code, and NAMING THE ARM IS ALREADY
# ITS JOB: it reports each stale or failed rule on its own line
# ("mmz: `just clippy` is never (never run)") before exiting. A `--status=json`
# pass here to rebuild that list was written first and deleted after measuring
# it — it printed every failing arm a second time, which is the noise this repo
# strips everywhere else. What is added below is only what mmz cannot know:
# which tree was asked, and what to do about the answer.
#
# The exit code also separates a stale or failed rule (1) from a selection
# holding no rule at all (7), and that case prints nothing of its own. A branch
# whose `.mmz/config.yaml` declares no gate-tagged rule has to be refused, not
# passed on the strength of having checked nothing.
gate=0
run_in_worktree mmz --is-fresh --tag gate || gate=$?
if [ "$gate" -ne 0 ]; then
    if [ "$gate" -eq 7 ]; then
        echo "integrate: $wt: mmz found no gate-tagged rule — that branch's .mmz/config.yaml declares none" >&2
    else
        echo "integrate: $wt: the arms above are not fresh, so the gate has not passed against '$branch'" >&2
    fi
    echo "  re-run \`just check\` in $wt, then integrate again" >&2
    exit 1
fi

# `-q` because a diffstat of the whole branch is not the answer to "did this
# land". The sha below is.
git merge --ff-only -q "$branch"

# The fast-forward invalidated this checkout's dev mt: the shell's PATH serves
# `target/mt-dev/debug/mt`, built from the pre-merge tree. Rebuilding here,
# rather than at the next `mt` call, is the whole exchange this line makes. A
# failed build does not un-land the merge, so it is named, never fatal.
if ! (cd "$here" && just build-mt); then
    echo "integrate: merged, but the dev mt did not rebuild — run \`just build-mt\` before typing mt" >&2
fi

git worktree remove "$wt"
echo "integrate: $wt: gate fresh; $onto fast-forwarded to $(git rev-parse --short HEAD), worktree removed"
