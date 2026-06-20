#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Drop broad dirty-tree gate from rm; add per-file pre-flight",
  status: done(2026, 6, 20),
  priority: ice(impact: 4, confidence: 0.9, ease: 7),
  estimate: (hours: 2),
  depends-on: (link("rm-targeted-removal.typ"),),
  tags: ("cli", "substrate"),
)

== Summary

Drop the broad dirty-tree gate from `rm`; replace with a per-file pre-flight
that checks only the task files being removed and lists all offenders at once.

== Problem

`rm` currently refuses with "working tree has uncommitted changes; rm rewrites
files in place — commit or stash first" whenever _any_ file in the repo is
dirty, not just the files being removed. The broad gate is unnecessary:

- Correctness is enforced per-file at mint time (`swhid_mint::git::mint_rev_path`
  already returns `Error::Dirty` / `Error::NotTracked` when a to-be-retired
  file's working tree bytes diverge from the last-touch commit, or the file
  has no commit at all).
- Unrelated dirty files (notes, drafts, other tasks) have no effect on
  tombstone soundness.
- Link rewrites in dirty survivors write new bytes on top of existing local
  edits, but the survivor is not pinned — its edit is the user's to review,
  same as any other non-rm change.

The original broad gate was inherited from `cleanup` as a UX nicety ("removal
is the only change to review"), but it blocks every legitimate workflow where
the working tree is not pristine.

== Fix

- Remove the `!git::is_clean(&root)` gate from `rm::run`.
- Before the real run (not dry-run), run a pre-flight that checks every
  eligible task file individually: `swhid_mint::git::mint_rev_path` per file,
  collecting _all_ failures so the user sees every offender in one shot.
- Emit task-centric, actionable error messages (slug + path + reason).
  Current message ("working tree has uncommitted changes…") gives no hint
  which file is the problem or why tombstones require a commit.

== Scope

- `crates/mindtape-cli/Cargo.toml`: add `swhid-mint` as a direct dependency.
- `crates/mindtape-cli/src/rm.rs`: remove broad gate; add `check_eligible_files`.
- `crates/mindtape-cli/src/rm_tests.rs`: tests for new pre-flight behaviour.
  - Unrelated dirty file + clean eligible task → succeeds.
  - Dirty eligible task → fails with task-centric message.
  - Untracked eligible task → fails with task-centric message.
  - Multiple offenders → all listed at once.
