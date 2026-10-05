#!/usr/bin/env bash
# .just/scripts/fork.sh — the body of the `fork` recipe
# (truth/tasks/add-a-just-fork-recipe-that-stands-up-a-ready-worktree.typ).
# Runs from the repo root, like the recipe that invokes it.
#
# What this is for, and it is not speed. Standing up a worktree for a delegated
# agent is four steps — `git worktree add`, `cd`, `direnv allow`, then assert the
# base is the commit the orchestrator meant — and only the last one is load-
# bearing. It is also the one always skipped, and the only one whose failure is
# silent: a worktree branched off the wrong base ships a diff that passes its own
# gate while dropping everything the real base carried.
#
# Warming the build cache is deliberately absent, and was the assumed motivation
# until it was measured. The dev shell inherits `kache` as `RUSTC_WRAPPER` from
# the host session, at a 99.7% hit rate over a 50 GiB content-addressed store, so
# a fresh worktree's `cargo build` is mostly hits already. Copying `target/` is
# the wrong lever regardless: this checkout's is 17 GB on ext4, where
# `cp --reflink` fails outright, so a seed is a real 17 GB write per worktree —
# and `cp -al` would let the worktree's build corrupt the source tree's cache.
# A cold `.mmz/cache/` is the same kind of correct: every gate arm running once
# in a tree no arm has ever run in is the honest answer, not a cost to paper
# over. What is left is ergonomics plus one assertion, plus the one `cargo
# build` at the end: the worktree's own dev `mt`, which its shell's PATH
# promises and kache serves at the same 99.7% hit rate as everything else.
#
# What the assertion can and cannot prove. It can prove the worktree landed on
# the commit that was named — the base is resolved to a full sha once, up front,
# and `git worktree add` is handed that sha rather than the symbolic name, so a
# branch that moves between resolving and creating cannot change the answer, and
# the created worktree's own HEAD is read back and compared. It cannot prove the
# named commit is the one the orchestrator meant: the recipe has no access to
# that intent. So the last thing it prints is the assertion line to paste into
# the agent's prompt, which is how the intent crosses the gap — the check on this
# side makes the sha trustworthy, the check on that side makes it the right one.
#
# Git's own refusals are not re-implemented. An existing branch name, an existing
# target directory, a base that is not a commit — git says so, by name, and a
# second copy of those messages here would only drift from them.
set -eo pipefail

slug="$1"
base="${2:-HEAD}"

# Reached through `--git-common-dir` rather than `$(pwd)`, so forking from a
# worktree still lands the new one beside the main checkout instead of nesting a
# `mindtape-a-b` beside `mindtape-a`. Siblings, never children: a worktree inside
# the repo would put a second `truth/tasks/` and a second `crates/` under every
# path-walking tool here — `mt check` would see each task stem twice, jscpd would
# report every file as a duplicate of itself, and typstyle would format a tree
# the branch does not own.
main_root=$(dirname "$(git rev-parse --path-format=absolute --git-common-dir)")
fork_root="${MINDTAPE_FORK_ROOT:-$(dirname "$main_root")}"
path="$fork_root/$(basename "$main_root")-$slug"

if ! base_sha=$(git rev-parse --verify --quiet "$base^{commit}"); then
    echo "fork: base '$base' does not resolve to a commit" >&2
    exit 1
fi
head_sha=$(git rev-parse --verify HEAD)

# The one refusal, and it is the dirty-tree case rather than the wrong-commit
# one. `git worktree add` copies commits, never the index or the working tree, so
# forking off a dirty HEAD hands the agent a base missing exactly the changes the
# caller is looking at while typing the command — the silent drop this recipe
# exists to prevent, in its most reachable form. Only when the base is HEAD:
# naming any other base is a statement that the working tree is not the subject.
#
# Untracked files count, which `--untracked-files=no` would have excluded. A
# brand-new source file is the change a fork drops most completely — there is no
# committed version of it to fall back to — and the noise argument does not
# apply here, because `git status --porcelain` already honours `.gitignore` and
# every regenerable tree in this repo (`target/`, `.mmz/cache/`, the generated
# fixture) is ignored. Found by running this recipe while fork.sh itself was
# untracked, and watching it report a clean HEAD.
if [ "$base_sha" = "$head_sha" ] && [ -z "$MINDTAPE_FORK_DIRTY_OK" ]; then
    dirty=$(git status --porcelain)
    if [ -n "$dirty" ]; then
        echo "fork: HEAD is dirty — the fork would silently drop these changes:" >&2
        sed 's/^/  /' <<<"$dirty" >&2
        echo "  commit them, or name an explicit base, or set MINDTAPE_FORK_DIRTY_OK=1" >&2
        exit 1
    fi
fi

# Reported, never refused. An explicitly named base is a decision, and forking
# off `main` from a branch that is ahead of it is a legitimate one — but it is
# also the exact shape of the mistake this recipe was filed after, so it is said
# out loud with both hashes rather than left to be noticed at integration.
if git merge-base --is-ancestor "$base_sha" "$head_sha"; then
    behind=$(git rev-list --count "$base_sha..$head_sha")
    if [ "$behind" -gt 0 ]; then
        echo "fork: base is $behind commit(s) behind this checkout's HEAD" >&2
        echo "  base $base_sha  HEAD $head_sha — the fork will not carry those commits" >&2
    fi
else
    echo "fork: base is NOT an ancestor of this checkout's HEAD" >&2
    echo "  base $base_sha  HEAD $head_sha — the fork starts on a divergent line" >&2
fi

git worktree add -b "$slug" "$path" "$base_sha"

# Read back rather than assumed. Everything above argues about what should land;
# this is the only line that knows what did.
wt_head=$(git -C "$path" rev-parse HEAD)
if [ "$wt_head" != "$base_sha" ]; then
    echo "fork: the new worktree is not at the base it was created from" >&2
    echo "  requested $base_sha  worktree HEAD $wt_head" >&2
    echo "  left in place for inspection — remove it with \`git worktree remove $path\`" >&2
    exit 1
fi
wt_branch=$(git -C "$path" symbolic-ref --short HEAD)
if [ "$wt_branch" != "$slug" ]; then
    echo "fork: the new worktree is on '$wt_branch', not '$slug'" >&2
    exit 1
fi

# Without this the worktree's `.envrc` is unauthorised and loads nothing there —
# every tool the gates run comes from the dev shell, so the failure reads as a
# missing binary rather than a missing allow. Silently, either way: a shell that
# loads .envrc on `cd` reports the refusal, but one driven by direnv's own
# export (`/etc/bash_env` on the NixOS host) discards it.
#
# And then the shell is loaded, not merely authorised. `direnv allow` records
# consent; the first load after it pays the flake evaluation and any
# substituter fetch the pin needs — measured at roughly two minutes on a warm
# store for this repo. Paying it here, once, visibly, is worth more than it
# looks: unpaid, it lands inside the delegated agent's first gate run, where a
# long foreground command is exactly what the harness auto-backgrounds, and an
# auto-backgrounded gate is how an agent ends its turn on a half-finished diff.
if command -v direnv >/dev/null 2>&1; then
    direnv allow "$path"
    echo "fork: loading the worktree's dev shell (first evaluation, once per worktree)" >&2
    direnv exec "$path" true

    # Then the worktree's dev `mt` is built once, because the shell's PATH
    # serves `$path/target/mt-dev/debug/mt` and this worktree has none. Built
    # here, visibly, for the same reason the shell is loaded here: the
    # delegated agent's first `mt` works, and a cold build does not land inside
    # its first gate run. Through the worktree's own recipe, so a base from
    # before `just build-mt` existed still forks with the hint instead of a
    # failure — its dev shell has no PATH entry for the binary either.
    if (cd "$path" && direnv exec . just --show build-mt) >/dev/null 2>&1; then
        (cd "$path" && direnv exec . just build-mt)
    else
        echo "fork: this base predates \`just build-mt\`; no dev mt was built" >&2
    fi
else
    echo "fork: direnv is not on PATH — the worktree's dev shell is NOT authorised" >&2
    echo "  run \`direnv allow\` in $path before running anything from the dev shell" >&2
fi

echo
echo "worktree  $path"
echo "branch    $wt_branch"
echo "base      $base_sha ($base)"

# A fork's `mt` and `check-examples` could read whichever checkout last ran
# `just install-lib`, not the lib beside them, so no note about the
# machine-global `@local/mindtape` symlink is needed. The loader auto-registers
# `<root>/lib` for the repo itself, both example fixtures declare a
# `[[typst.local-packages]]` entry pointing at it, and `just check-examples`
# runs with the global directory emptied so the entry cannot rot. A worktree
# reads its own prelude with nothing printed and nothing to remember.
echo
echo "pin the base in the agent's prompt, and have it assert before any work:"
echo "  cd $path"
echo "  git merge-base --is-ancestor $base_sha HEAD && [ \"\$(git rev-parse HEAD)\" = $base_sha ]"
