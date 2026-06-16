# 3. SWHID cleanup tombstones and the shared edit substrate

Status: Accepted
Date: 2026-06-16

## Context

MindTape wants `cleanup`: remove tasks in a terminal status and rewrite every
surviving link to a removed task so nothing dangles. speconaut already ships this
as `cleanup` plus a `git-ref("path", "commit")` tombstone (URL scheme
`speconaut+git://`), backed by bespoke byte-scanning rewriters and git resolution.

ADR 002 mandates one shared eval/edit substrate across both tools; two trackers
must not carry two cleanup implementations. The operator wants a standard,
self-describing identifier for retired artifacts rather than the ad-hoc `git-ref`
DSL helper, and wants one identifier scheme spanning content/dir/rev across tools.

## Decision

1. Tombstone format: a plain Typst `link("swh:1:rev:<full-40-hex-commit>;path=<root-rel-path>")[Label]`.
   SWHID (ISO/IEC 18670:2025), revision-scoped, qualified by path. No `origin`
   (one repo, uniform — a collector assumes the current git origin), no blob
   (used only to verify, not stored). Full 40-hex commit, never abbreviated.
   `rev` is the git commit object SHA-1; `;path=` names the file inside it.
   Recovery: `git show <commit>:<path>`. For cnt/dir/rev the SWHID hash is
   sha1_git, identical to git's object SHA-1, so git already computes it.

2. Three consumer-blind pieces, shared by both tools, hosted in MindTape's
   `crates/` for now (ADR 002 will later move them to their own repos):
   - `typst-edit` (new): parse-plane (`typst-syntax`) span queries plus an
     atomic, non-overlapping changeset. Replaces speconaut's byte-scan/raw-mask
     rewriters. Plan-then-apply: parse every affected file and locate every edit
     up front; a wrong-shaped file aborts the run before any byte is written.
   - `swhid-mint` (new): mint and resolve SWHIDs. The core is VCS-agnostic
     (render/parse); backends are cargo features — `git` (default) shells out to
     the `git` binary, other VCS later. SWHID format hand-rolled — for cnt/dir/rev
     the hash is sha1_git == git's own object SHA-1, so git computes it and we only
     prefix `swh:1:<kind>:`. Exposes the cnt/dir/rev scheme and a `resolves()`
     check used by each tool's `check` command (replaces speconaut's
     `check_git_refs`).
   - `cleanup` (thin): orchestration only — enumerate terminal tasks, confirm,
     mint tombstones, rewrite `link()` spans, delete files, single commit.
     Refuse on a dirty tree; resolve before deleting.

3. Editor substrate ratified as `typst-syntax` AST spans (extends the
   `typst-cst` plane), not byte-scan + raw-mask. Links inside raw blocks and
   escaped `\#link` are not function calls, so they are skipped for free — no
   textual masking. Wrong-shaped files fail loud; the user fixes and reruns.

4. `cleanup` is atomic: the full edit set across all files is validated before
   anything is written; any unresolved commit or unparseable file aborts with no
   changes.

5. speconaut retires `git-ref` / `speconaut+git://` in favour of SWHID links and
   adopts `typst-edit` + `swhid-mint`, shedding its bespoke scanners. Tracked as a
   speconaut task; it may lag this ADR.

6. The `swhid` crate is rejected as a dependency: its `git` feature pulls libgit2
   and targets archival SWHIDs of HEAD/tags/snapshots, not "a path at a commit
   plus a recoverability check"; its pure type drags `clap` and
   `sha1collisiondetection` we never use. The format is a stable ISO standard a
   short parser handles. Reconsider only if non-git intrinsic hashing (cnt/dir
   without git) or Software Heritage archive lookups are wanted.

## Consequences

- Gain: one cleanup substrate across both tools; a standard, self-describing
  retired-artifact identifier; ~1000+ lines of speconaut scanners become library
  calls; atomic, abort-safe cleanup.
- Cost: edits require a successful parse (acceptable — refuse wrong-shaped
  files). A new small crate (`swhid-mint`) is justified as the sharing unit across
  two repos despite its size, and because it carries the verify + resolve logic
  both tools need (not a ten-line wrapper).
- MindTape's link model must treat `swh:` targets as history pointers: skip
  stem-normalization, skip existence checks, allow a label.
- Resolved: crate is `swhid-mint`, VCS backends behind features (`git` default);
  `cnt` and `rev` minting are exposed, `dir`/`rel`/`snp` deferred until needed.

## Deprecation Criteria

Supersede when the shared crates move to their own repositories (ADR 002
follow-up) — update the crate-host references then. Revisit the tombstone format
if MindTape repos are deposited in Software Heritage (an `origin` qualifier
becomes worthwhile) or if non-git SWHID computation is needed (pull the `swhid`
crate).
